use std::{
    pin::Pin,
    task::{Context, Poll},
};

use actix_service::*;
use actix_web::dev::*;
use actix_web::{http::header::HeaderMap, Error};
use futures::{
    future::{ok, Ready},
    Future, FutureExt,
};
use opentelemetry::propagation::Extractor;
use tracing_batteries::prelude::*;

pub struct TracingLogger;

impl<S, B> Transform<S, ServiceRequest> for TracingLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TracingLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TracingLoggerMiddleware { service })
    }
}

#[doc(hidden)]
pub struct TracingLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TracingLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let user_agent = req
            .headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        let span = info_span!(
            "request",
            "otel.kind" = "server",
            "otel.name" = req.match_pattern().unwrap_or_else(|| req.uri().path().to_string()),
            "net.transport" = "IP.TCP",
            "net.peer.ip" = %req.connection_info().realip_remote_addr().unwrap_or(""),
            "http.target" = %req.uri(),
            "http.user_agent" = %user_agent,
            "http.status_code" = EmptyField,
            "http.method" = %req.method(),
            "http.url" = %req.match_pattern().unwrap_or_else(|| req.path().into()),
            "http.headers" = %req.headers().iter().map(|(k, v)| format!("{k}: {v:?}")).collect::<Vec<_>>().join("\n"),
        );

        // Propagate OpenTelemetry parent span context information
        let context = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderMapExtractor::from(req.headers()))
        });

        let _ = span.set_parent(context);

        let fut = self
            .service
            .call(req)
            .map(move |outcome| match &outcome {
                Ok(response) => {
                    Span::current()
                        .record("http.status_code", display(response.response().status()));
                    outcome
                }
                Err(error) => {
                    Span::current().record(
                        "http.status_code",
                        display(error.as_response_error().status_code()),
                    );
                    outcome
                }
            })
            .instrument(span);

        Box::pin(fut)
    }
}

struct HeaderMapExtractor<'a> {
    headers: &'a HeaderMap,
}

impl<'a> From<&'a HeaderMap> for HeaderMapExtractor<'a> {
    fn from(headers: &'a HeaderMap) -> Self {
        HeaderMapExtractor { headers }
    }
}

impl<'a> Extractor for HeaderMapExtractor<'a> {
    fn get(&self, key: &str) -> Option<&'a str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|v| v.as_str()).collect()
    }
}
