mod models;
mod state;
#[cfg(test)]
mod test;

use actix_web::{get, web};

pub use self::state::HealthState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health_v1).service(health_v2);
}

#[get("/api/v1/health")]
pub async fn health_v1(state: web::Data<state::HealthState>) -> models::HealthV1 {
    state.health()
}

#[get("/api/v2/health")]
pub async fn health_v2(state: web::Data<state::HealthState>) -> models::HealthV2 {
    state.health()
}
