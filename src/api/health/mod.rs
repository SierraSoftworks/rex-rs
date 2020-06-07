mod models;
#[cfg(test)]
mod test;

use actix_web::{get, web};
use crate::models::*;
use super::APIError;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health_v1)
        .service(health_v2);
}

#[get("/api/v1/health")]
pub async fn health_v1(state: web::Data<GlobalState>) -> Result<models::HealthV1, APIError> {
    state.store.send(GetHealth {}).await?.map(|health| health.clone().into())
}

#[get("/api/v2/health")]
pub async fn health_v2(state: web::Data<GlobalState>) ->Result<models::HealthV2, APIError> {
    state.store.send(GetHealth {}).await?.map(|health| health.clone().into())
}
