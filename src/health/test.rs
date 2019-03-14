use super::super::app;
use super::state;
use rocket::http::{ContentType, Status};
use rocket::local::Client;
use rocket_contrib::json;

#[test]
fn health_v1() {
    let client = Client::new(app()).expect("valid rocket instance");
    let mut response = client.get("/api/v1/health").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    assert_eq!(
        response.body_string(),
        Some(
            json!({
                "ok": true,
            })
            .to_string()
        )
    );
}

#[test]
fn health_v2() {
    let client = Client::new(app()).expect("valid rocket instance");
    let state: &state::HealthState = client.rocket().state().unwrap();

    let mut response = client.get("/api/v2/health").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    assert_eq!(
        response.body_string(),
        Some(
            json!({
                "ok": true,
                "started_at": state.started_at,
            })
            .to_string()
        )
    )
}
