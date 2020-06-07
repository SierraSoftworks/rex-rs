use super::{configure, models};
use crate::models::*;
#[cfg(test)]
use actix_web::{test, App};

#[actix_rt::test]
async fn health_v1_status() {
    let mut app =
        test::init_service(App::new().data(GlobalState::new()).configure(configure)).await;

    let req = test::TestRequest::with_uri("/api/v1/health").to_request();
    let response = test::call_service(&mut app, req).await;

    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn health_v1_content() {
    let state = GlobalState::new();

    let mut app = test::init_service(App::new().data(state.clone()).configure(configure)).await;

    let req = test::TestRequest::with_uri("/api/v1/health").to_request();
    let response: models::HealthV1 = test::read_response_json(&mut app, req).await;

    assert_eq!(response.ok, true);
}

#[actix_rt::test]
async fn health_v2_status() {
    let mut app =
        test::init_service(App::new().data(GlobalState::new()).configure(configure)).await;

    let req = test::TestRequest::with_uri("/api/v2/health").to_request();
    let response = test::call_service(&mut app, req).await;

    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn health_v2_content() {
    let state = GlobalState::new();

    let mut app = test::init_service(App::new().data(state.clone()).configure(configure)).await;

    let req = test::TestRequest::with_uri("/api/v2/health").to_request();
    let response: models::HealthV2 = test::read_response_json(&mut app, req).await;

    assert_eq!(response.ok, true);
    assert_eq!(response.started_at, state.store.send(GetHealth {}).await.expect("the actor should respond").expect("we should get the health").started_at);
}
