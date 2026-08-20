//! Test harness shared by the handler tests.
//!
//! Handler tests run against [`MemoryStore`] with authentication switched off,
//! so every request arrives as the local developer (principal `0`). That keeps
//! them about handlers. Token validation is covered separately, in [`oidc`],
//! against a real OIDC provider served by wiremock — so the code path that
//! ships is the code path under test, and no test-only weakening of the
//! verifier exists to be shipped by accident.

#[macro_use]
mod macros;

pub mod oidc;

use std::sync::Arc;

use actix_web::{App, test};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::{auth::OidcState, config::Config, db::MemoryStore, services::ServicesContainer, web};

// Re-exported so that a test module's `use crate::testing::*` brings in
// everything it needs to poke at the store behind a services container.
pub use crate::{db::Store, services::Services};

pub type TestServices = ServicesContainer<MemoryStore>;

pub fn test_log_init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

/// A services container with an empty in-memory store and no identity provider.
pub fn test_services() -> TestServices {
    services_for(Config::default())
}

pub fn services_for(config: Config) -> TestServices {
    let config = Arc::new(config);
    let oidc = Arc::new(OidcState::new(&config).expect("the OIDC state should build"));

    ServicesContainer::new(config, MemoryStore::new(), oidc)
}

pub async fn get_test_app(
    services: TestServices,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let oidc = crate::services::Services::oidc(&services).clone();

    test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(services))
            .app_data(actix_web::web::Data::new(oidc))
            .configure(web::api::configure::<TestServices>),
    )
    .await
}

/// The header the handler tests send.
///
/// With no identity provider configured the server never looks at it, but
/// sending it keeps the tests honest about what a real request carries.
pub fn auth_token() -> String {
    "Bearer test-token".to_string()
}

pub fn assert_location_header(headers: &actix_web::http::header::HeaderMap, prefix: &str) {
    let location = headers
        .get("Location")
        .expect("a location header")
        .to_str()
        .expect("a non-empty location header");

    debug!("Got location header: {}", location);

    assert!(
        location.contains(prefix),
        "expected the location header to contain `{prefix}`, got `{location}`"
    );

    let id =
        String::from(&location[location.find(prefix).expect("index of path") + prefix.len()..]);
    assert_ne!(id, "");
}

pub async fn assert_status(
    resp: actix_web::dev::ServiceResponse,
    expected_status: actix_web::http::StatusCode,
) -> actix_web::dev::ServiceResponse {
    if expected_status != resp.status() {
        let status = resp.status();
        let err: rex_api::ApiError = get_content(resp).await;
        panic!(
            "Unexpected response code (got == expected)\n  got: {status}\n  expected: {expected_status}\n  error: {err}"
        )
    } else {
        resp
    }
}

pub async fn get_content<T: DeserializeOwned>(resp: actix_web::dev::ServiceResponse) -> T {
    test::read_body_json(resp).await
}
