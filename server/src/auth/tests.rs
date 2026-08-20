//! Token validation, tested against a real OIDC provider.
//!
//! Everything here goes through the same verifier the server uses in
//! production: real discovery, real JWKS, real RS256 signatures. There is no
//! test-only switch that relaxes verification, which is the point — the
//! previous implementation had one, and a codebase that can be told not to
//! check signatures eventually is.

use std::collections::HashMap;

use rex_api::{AuthPrincipal, HealthV1};
use serde_json::json;

use crate::{
    config::{Config, OidcConfig},
    testing::{get_test_app, oidc::TestIdentityProvider, services_for, test_log_init},
};

const CLIENT_ID: &str = "rex-test-client";

/// A principal id in the shape Azure AD's `oid` claim produces.
const PRINCIPAL: &str = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
const PRINCIPAL_ID: &str = "6ba7b8109dad11d180b400c04fd430c8";

fn config_for(endpoint: String, user_acl: Option<&str>) -> Config {
    let mut config = Config::default();

    config.web.auth.user_acl = user_acl.map(String::from);
    config.web.auth.oidc = Some(OidcConfig {
        endpoint,
        client_id: CLIENT_ID.into(),
        client_secret: None,
        scopes: vec!["openid".into(), "profile".into(), "email".into()],
        username_claim: "oid".into(),
        email_claim: "email".into(),
        discovery_ttl_seconds: 3600,
    });

    config
}

fn claims() -> HashMap<&'static str, serde_json::Value> {
    HashMap::from([
        ("oid", json!(PRINCIPAL)),
        ("email", json!("testy@example.com")),
        ("name", json!("Testy McTesterson")),
    ])
}

async fn call(config: Config, bearer: Option<&str>) -> actix_web::dev::ServiceResponse {
    let app = get_test_app(services_for(config)).await;

    let mut req = actix_web::test::TestRequest::with_uri("/api/v1/auth/me")
        .method(actix_web::http::Method::GET);

    if let Some(bearer) = bearer {
        req = req.insert_header(("Authorization", format!("Bearer {bearer}")));
    }

    actix_web::test::call_service(&app, req.to_request()).await
}

#[actix_rt::test]
async fn a_valid_token_identifies_its_principal() {
    test_log_init();

    let idp = TestIdentityProvider::start(CLIENT_ID).await;
    let token = idp.id_token(PRINCIPAL, claims());

    let response = call(config_for(idp.endpoint(), None), Some(&token)).await;
    assert_eq!(response.status(), actix_web::http::StatusCode::OK);

    let principal: AuthPrincipal = actix_web::test::read_body_json(response).await;

    assert_eq!(
        principal.id, PRINCIPAL_ID,
        "the configured username claim should decide the principal id"
    );
    assert_eq!(principal.email, "testy@example.com");
    assert_eq!(principal.name, "Testy McTesterson");
}

#[actix_rt::test]
async fn a_missing_token_is_unauthorized() {
    test_log_init();

    let idp = TestIdentityProvider::start(CLIENT_ID).await;

    let response = call(config_for(idp.endpoint(), None), None).await;
    assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn an_expired_token_is_rejected() {
    test_log_init();

    let idp = TestIdentityProvider::start(CLIENT_ID).await;
    let token = idp.expired_id_token(PRINCIPAL);

    let response = call(config_for(idp.endpoint(), None), Some(&token)).await;
    assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn a_token_for_another_audience_is_rejected() {
    test_log_init();

    let idp = TestIdentityProvider::start("somebody-else").await;
    let token = idp.id_token(PRINCIPAL, claims());

    let response = call(config_for(idp.endpoint(), None), Some(&token)).await;
    assert_eq!(
        response.status(),
        actix_web::http::StatusCode::UNAUTHORIZED,
        "a token minted for a different client must not be accepted"
    );
}

/// The algorithm-confusion attack: sign with HMAC using the provider's public
/// modulus as the key, and hope the verifier treats a public value as a secret.
#[actix_rt::test]
async fn an_hmac_signed_token_is_rejected() {
    test_log_init();

    let idp = TestIdentityProvider::start(CLIENT_ID).await;
    let token = idp.hmac_forged_token(PRINCIPAL);

    let response = call(config_for(idp.endpoint(), None), Some(&token)).await;
    assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn a_token_missing_the_username_claim_is_rejected() {
    test_log_init();

    let idp = TestIdentityProvider::start(CLIENT_ID).await;
    let token = idp.id_token(PRINCIPAL, HashMap::new());

    let response = call(config_for(idp.endpoint(), None), Some(&token)).await;
    assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn the_user_acl_gates_sign_in() {
    test_log_init();

    let idp = TestIdentityProvider::start(CLIENT_ID).await;
    let token = idp.id_token(PRINCIPAL, claims());

    let allowed = config_for(
        idp.endpoint(),
        Some(r#"claims.email endswith "@example.com""#),
    );
    assert_eq!(
        call(allowed, Some(&token)).await.status(),
        actix_web::http::StatusCode::OK
    );

    let denied = config_for(
        idp.endpoint(),
        Some(r#"claims.email endswith "@example.org""#),
    );
    assert_eq!(
        call(denied, Some(&token)).await.status(),
        actix_web::http::StatusCode::FORBIDDEN,
        "a valid token from outside the ACL should be refused"
    );
}

#[actix_rt::test]
async fn the_api_refuses_unauthenticated_callers() {
    test_log_init();

    let idp = TestIdentityProvider::start(CLIENT_ID).await;
    let app = get_test_app(services_for(config_for(idp.endpoint(), None))).await;

    let response = actix_web::test::call_service(
        &app,
        actix_web::test::TestRequest::with_uri("/api/v3/ideas").to_request(),
    )
    .await;

    assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

/// Health has to answer before anybody has signed in, or the readiness probe
/// can never pass.
#[actix_rt::test]
async fn health_stays_anonymous() {
    test_log_init();

    let idp = TestIdentityProvider::start(CLIENT_ID).await;
    let app = get_test_app(services_for(config_for(idp.endpoint(), None))).await;

    let response = actix_web::test::call_service(
        &app,
        actix_web::test::TestRequest::with_uri("/api/v1/health").to_request(),
    )
    .await;

    assert_eq!(response.status(), actix_web::http::StatusCode::OK);

    let health: HealthV1 = actix_web::test::read_body_json(response).await;
    assert!(health.ok);
}

#[actix_rt::test]
async fn metadata_describes_the_provider() {
    test_log_init();

    let idp = TestIdentityProvider::start(CLIENT_ID).await;
    let app = get_test_app(services_for(config_for(idp.endpoint(), None))).await;

    let response = actix_web::test::call_service(
        &app,
        actix_web::test::TestRequest::with_uri("/api/v1/auth/metadata").to_request(),
    )
    .await;

    assert_eq!(response.status(), actix_web::http::StatusCode::OK);

    let metadata: rex_api::AuthMetadata = actix_web::test::read_body_json(response).await;
    assert!(metadata.enabled);
    assert_eq!(metadata.client_id.as_deref(), Some(CLIENT_ID));
    assert_eq!(
        metadata.authorization_endpoint,
        Some(format!("{}/authorize", idp.endpoint()))
    );
}

#[actix_rt::test]
async fn metadata_reports_authentication_being_disabled() {
    test_log_init();

    let app = get_test_app(services_for(Config::default())).await;

    let response = actix_web::test::call_service(
        &app,
        actix_web::test::TestRequest::with_uri("/api/v1/auth/metadata").to_request(),
    )
    .await;

    let metadata: rex_api::AuthMetadata = actix_web::test::read_body_json(response).await;
    assert!(!metadata.enabled);
    assert_eq!(metadata.client_id, None);
}
