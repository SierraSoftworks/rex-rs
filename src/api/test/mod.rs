use crate::api::configure;
use crate::models::*;
use actix_web::{test, web, App};
use oidc::token::Jws;
use web::BytesMut;
use serde::{de::DeserializeOwned};
use futures::StreamExt;

pub fn test_log_init() {
    let _ = env_logger::builder().is_test(true).filter_level(log::LevelFilter::Debug).try_init();
}

pub async fn get_test_app(state: GlobalState) -> impl actix_web::dev::Service<Request = actix_http::Request, Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>, Error = actix_web::Error> {
    test::init_service(
        App::new()
            .data(state.clone())
            .configure(configure),
    )
    .await
}

pub fn assert_location_header(header: &actix_web::http::HeaderMap, prefix: &str) {
    let location = header.get("Location")
        .expect("a location header")
        .to_str()
        .expect("a non-empty location header");

    debug!("Got location header: {}", location);

    assert!(location.contains(prefix));

    let id = String::from(
        &location[location.find(prefix).expect("index of path") + prefix.len()..],
    )
    .clone();
    assert_ne!(id, "");
}

pub fn auth_token() -> String {
    let token = Jws::new_decoded(biscuit::jws::Header {
        registered: biscuit::jws::RegisteredHeader {
            algorithm: biscuit::jwa::SignatureAlgorithm::HS256,
            ..Default::default()
        },
        private: biscuit::Empty{},
    }, crate::api::AuthToken {
        aud: "https://test.example.com".into(),
        exp: 0,
        iat: 0,
        name: "Testy McTesterson".into(),
        oid: "00000000-0000-0000-0000-000000000000".into(),
        scp: "".into(),
        sub: "testy@example.com".into(),
        roles: vec![],
        unique_name: "testy@example.com".into(),
        ..Default::default()
    });

    let content = token.encode(&biscuit::jws::Secret::bytes_from_str("test")).unwrap().unwrap_encoded().to_string();

    "Bearer ".to_string() + content.as_str()
}

pub async fn assert_status(resp: &mut actix_web::dev::ServiceResponse, expected_status: http::StatusCode) {
    if expected_status == resp.status() {
        return
    }

    let err: super::APIError = get_content(resp).await;
    panic!("Unexpected response code (got == expected)\n  got: {}\n  expected: {}\n  error: {}", resp.status(), expected_status, err)
}

pub async fn get_content<T: DeserializeOwned>(resp: &mut actix_web::dev::ServiceResponse) -> T {
    let mut body = resp.take_body();
    let mut bytes = BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }
    let content_bytes = bytes.freeze();

    serde_json::from_slice(&content_bytes)
        .unwrap_or_else(|err| {
            panic!("Failed to deserialize response: {}", err);
        })
}