use std::{
    pin::Pin,
    task::{Context, Poll},
};

use actix_service::*;
use actix_web::dev::*;
use actix_web::{Error, http::header::HeaderMap};
use futures::{
    Future, FutureExt,
    future::{Ready, ok},
};
use opentelemetry::propagation::Extractor;
use tracing_batteries::prelude::*;

/// Headers whose values must never reach a span.
///
/// Recording headers wholesale is genuinely useful for debugging and genuinely
/// dangerous: `Authorization` carries the bearer token that *is* the session.
const REDACTED_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "proxy-authorization",
];

/// Query parameters whose values must never reach a span, for the same reason.
const REDACTED_QUERY_PARAMS: &[&str] = &["code", "state", "token", "id_token", "refresh_token"];

const REDACTED: &str = "[redacted]";

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
            .unwrap_or("")
            .to_string();

        let span = info_span!(
            "request",
            "otel.kind" = "server",
            "otel.name" = req.match_pattern().unwrap_or_else(|| req.uri().path().to_string()),
            "net.transport" = "IP.TCP",
            "net.peer.ip" = %req.connection_info().realip_remote_addr().unwrap_or(""),
            "http.target" = %redact_query(&req.uri().to_string()),
            "http.user_agent" = %user_agent,
            "http.status_code" = EmptyField,
            "http.method" = %req.method(),
            "http.url" = %req.match_pattern().unwrap_or_else(|| req.path().into()),
            "http.headers" = %redact_headers(req.headers()),
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

fn redact_headers(headers: &HeaderMap) -> String {
    headers
        .iter()
        .map(|(name, value)| {
            if REDACTED_HEADERS.contains(&name.as_str()) {
                format!("{name}: {REDACTED}")
            } else {
                format!("{name}: {value:?}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Rewrites a URL's query string, replacing the values of sensitive parameters.
///
/// The auth callback lands with the authorization `code` in the query string,
/// and that code is worth a session to whoever reads the trace.
fn redact_query(uri: &str) -> String {
    let Some((path, query)) = uri.split_once('?') else {
        return uri.to_string();
    };

    let redacted = query
        .split('&')
        .map(|pair| match pair.split_once('=') {
            Some((name, _)) if REDACTED_QUERY_PARAMS.contains(&name) => {
                format!("{name}={REDACTED}")
            }
            _ => pair.to_string(),
        })
        .collect::<Vec<_>>()
        .join("&");

    format!("{path}?{redacted}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_sensitive_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            actix_web::http::header::AUTHORIZATION,
            "Bearer super-secret".parse().unwrap(),
        );
        headers.insert(
            actix_web::http::header::USER_AGENT,
            "rex-tests".parse().unwrap(),
        );

        let rendered = redact_headers(&headers);
        assert!(
            !rendered.contains("super-secret"),
            "the bearer token must not reach a span: {rendered}"
        );
        assert!(rendered.contains("rex-tests"));
    }

    #[test]
    fn redacts_sensitive_query_parameters() {
        let redacted = redact_query("/auth/callback?code=super-secret&state=abc&next=/collections");

        assert!(!redacted.contains("super-secret"));
        assert!(!redacted.contains("abc"));
        assert!(redacted.contains("next=/collections"));
    }

    #[test]
    fn leaves_urls_without_a_query_string_alone() {
        assert_eq!(redact_query("/api/v3/ideas"), "/api/v3/ideas");
    }
}
