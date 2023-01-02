use actix_web::{get, http::header::ContentType, web, HttpRequest, HttpResponse};
use http::HeaderValue;
use tracing::{field, instrument, Span};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_ui_path);
}

static DEFAULT_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/octet-stream");

#[instrument(
    skip(req),
    fields(
        otel.kind="client",
        http.method="GET",
        http.target="/{ui_path}",
        http.path=%req.path(),
        http.user_agent=%req.headers().get("User-Agent").map(|h| h.to_str().unwrap_or("")).unwrap_or(""),
        http.status_code=field::Empty))]
#[get("/{ui_path:.*}")]
pub async fn get_ui_path(req: HttpRequest) -> HttpResponse {
    match std::env::var("INTERFACE_BACKEND_URI").ok() {
        Some(uri) if !uri.is_empty() => {
            let client = reqwest::Client::new();
            let res = client
                .get(&format!("{}/{}", uri, req.match_info().query("ui_path")))
                .send()
                .await;
            match res {
                Ok(res) => {
                    let status = res.status();
                    let content_type = res
                        .headers()
                        .get("content-type")
                        .unwrap_or(&DEFAULT_CONTENT_TYPE)
                        .clone();
                    Span::current().record("http.status_code", status.as_u16());

                    match res.bytes().await {
                        Ok(bytes) => HttpResponse::build(status)
                            .content_type(content_type)
                            .body(bytes),
                        Err(err) => {
                            error!({ exception.message = %err }, "Failed to read response body.");
                            HttpResponse::GatewayTimeout()
                                .body("Encountered an internal error while fetching the UI, please try again later.")
                        }
                    }
                }
                Err(err) => {
                    error!({ exception.message = %err }, "Request to UI backend failed.");
                    HttpResponse::ServiceUnavailable()
                        .content_type(ContentType::plaintext())
                        .body("Unable to retrieve the UI, please try again later.")
                }
            }
        }
        _ => {
            warn!("No backend has been configured for the UI.");
            HttpResponse::NotFound()
                .content_type(ContentType::plaintext())
                .body("No UI backend has been configured for Rex, the UI will be disabled.")
        }
    }
}
