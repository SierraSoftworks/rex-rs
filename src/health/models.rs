use super::super::api;
use super::state;

use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};

#[derive(Serialize, Deserialize)]
pub struct HealthV1 {
    pub ok: bool,
}

impl api::StateView<state::HealthState> for HealthV1 {
    fn to_state(&self) -> state::HealthState {
        state::HealthState {
            ok: self.ok,
            started_at: chrono::Utc::now(),
        }
    }

    fn from_state(state: &state::HealthState) -> Self {
        HealthV1 { ok: state.ok }
    }
}

impl Responder for HealthV1 {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(&self)))
    }
}

#[derive(Serialize, Deserialize)]
pub struct HealthV2 {
    pub ok: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl api::StateView<state::HealthState> for HealthV2 {
    fn to_state(&self) -> state::HealthState {
        state::HealthState {
            ok: self.ok,
            started_at: self.started_at,
        }
    }

    fn from_state(state: &state::HealthState) -> Self {
        HealthV2 {
            ok: state.ok,
            started_at: state.started_at,
        }
    }
}

impl Responder for HealthV2 {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(&self)))
    }
}
