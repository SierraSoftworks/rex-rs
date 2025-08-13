use crate::api::configure;
use crate::models::*;
use actix_web::{
    test::{self, read_body_json},
    App,
};
use chrono::{Duration, Utc};
use openidconnect::{
    core::{CoreHmacKey, CoreJwsSigningAlgorithm},
    Audience, EndUserName, IssuerUrl, LocalizedClaim, StandardClaims, SubjectIdentifier,
};
use serde::de::DeserializeOwned;

use super::auth::{AuthAdditionalClaims, AuthIdToken, AuthIdTokenClaims};

pub fn test_log_init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

pub async fn get_test_app(
    state: GlobalState,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(state.clone()))
            .configure(configure),
    )
    .await
}

pub fn assert_location_header(header: &actix_web::http::header::HeaderMap, prefix: &str) {
    let location = header
        .get("Location")
        .expect("a location header")
        .to_str()
        .expect("a non-empty location header");

    debug!("Got location header: {}", location);

    assert!(location.contains(prefix));

    let id =
        String::from(&location[location.find(prefix).expect("index of path") + prefix.len()..]);
    assert_ne!(id, "");
}

pub fn auth_token() -> String {
    let mut localized_name = LocalizedClaim::new();
    localized_name.insert(None, EndUserName::new("Testy McTesterson".to_string()));

    let token = AuthIdToken::new(
        AuthIdTokenClaims::new(
            IssuerUrl::new("https://auth.example.com".to_string()).expect("Issuer should always parse correctly."),
            vec![Audience::new("https://test.example.com".to_string())],
            Utc::now() + Duration::seconds(300),
            Utc::now(),
            StandardClaims::new(
                SubjectIdentifier::new("testy@example.com".to_string()),
            ).set_name(Some(localized_name)),
            AuthAdditionalClaims {
                oid: "00000000-0000-0000-0000-000000000000".into(),
                scp: "Ideas.Read Ideas.Write Collections.Read Collections.Write RoleAssignments.Write Users.Read".into(),
                roles: vec!["Administrator".into()],
                unique_name: "testy@example.com".into(),
            }
        ),
        &CoreHmacKey::new("test"),
        CoreJwsSigningAlgorithm::HmacSha256,
        None,
        None,
    ).expect("The token should be generated correctly");

    format!("Bearer {}", token.to_string())
}

pub async fn assert_status(
    resp: actix_web::dev::ServiceResponse,
    expected_status: http::StatusCode,
) -> actix_web::dev::ServiceResponse {
    if expected_status != resp.status() {
        let status = resp.status();
        let err: super::APIError = get_content(resp).await;
        panic!(
            "Unexpected response code (got == expected)\n  got: {}\n  expected: {}\n  error: {}",
            status, expected_status, err
        )
    } else {
        resp
    }
}

pub async fn get_content<T: DeserializeOwned>(resp: actix_web::dev::ServiceResponse) -> T {
    read_body_json(resp).await
}
