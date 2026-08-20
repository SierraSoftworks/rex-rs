//! Serving the web UI out of the binary itself.
//!
//! Trunk builds the UI into `ui/dist`, which `include_dir!` embeds at compile
//! time — that embedding is the whole reason a single file is all a deployment
//! needs. Anything that is not an embedded asset falls through to `index.html`
//! so the client-side router can take it from there.

use actix_web::{
    HttpRequest, HttpResponse,
    http::header::{CacheControl, CacheDirective, ContentType},
    web,
};
use include_dir::{Dir, include_dir};
use tracing::instrument;

static UI: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../ui/dist");

pub fn configure_ui(cfg: &mut web::ServiceConfig) {
    cfg.default_service(web::route().to(serve_ui));
}

#[instrument(
    name = "ui.serve",
    skip(req),
    fields(otel.kind = "server", http.path = %req.path()))]
async fn serve_ui(req: HttpRequest) -> HttpResponse {
    let path = req.path().trim_start_matches('/');

    if let Some(file) = UI.get_file(path) {
        // Trunk fingerprints its output, so anything it emitted can be cached
        // indefinitely; the document itself must not be.
        return asset_response(path, file.contents(), true);
    }

    match UI.get_file("index.html") {
        Some(index) => asset_response("index.html", index.contents(), false),
        None => {
            // The binary was built before `trunk build` ran. The build script
            // warns about this, but saying so here as well saves somebody a
            // confusing few minutes.
            HttpResponse::InternalServerError()
                .content_type(ContentType::plaintext())
                .body(
                    "The web UI was not built into this binary. Run `trunk build` in `ui/` and rebuild the server.",
                )
        }
    }
}

fn asset_response(path: &str, body: &'static [u8], immutable: bool) -> HttpResponse {
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let cache = if immutable {
        CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(31_536_000),
            CacheDirective::Extension("immutable".into(), None),
        ])
    } else {
        CacheControl(vec![CacheDirective::NoCache])
    };

    HttpResponse::Ok()
        .content_type(mime.as_ref())
        .insert_header(cache)
        .body(body)
}
