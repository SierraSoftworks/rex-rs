//! The HTTP surface: the JSON API, the auth broker endpoints, and the embedded
//! web UI.

pub mod api;
mod responder;
mod ui;

pub use responder::{ApiLocation, ApiResponse};
pub use ui::configure_ui;

use actix_web::web;

use crate::services::Services;

/// Registers everything the server serves, in the order routes should be
/// matched: the API first, then the UI's own assets, then the single-page
/// fallback.
pub fn configure<S: Services>(cfg: &mut web::ServiceConfig) {
    api::configure::<S>(cfg);
    configure_ui(cfg);
}
