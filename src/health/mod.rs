mod models;
mod state;
#[cfg(test)]
mod test;

use actix_web::{App, Json, pred};
use crate::stator::Stator;

pub use self::state::new_state;

pub fn configure(app: App<std::sync::Arc<::state::Container>>) -> App<std::sync::Arc<::state::Container>> {
    app
        .resource("/api/v1/health", |r| r.route().filter(pred::Get()).with(health_v1))
        .resource("/api/v2/health", |r| r.route().filter(pred::Get()).with(health_v2))
}

pub fn health_v1(state: Stator<state::HealthState>) -> Json<models::HealthV1> {
    Json(state::health(&state))
}

pub fn health_v2(state: Stator<state::HealthState>) -> Json<models::HealthV2> {
    Json(state::health(&state))
}
