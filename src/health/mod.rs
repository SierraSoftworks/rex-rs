use rocket::State;
use rocket_contrib::json::Json;

mod models;
mod state;
#[cfg(test)]
mod test;

pub use state::new_state;

#[get("/api/v1/health")]
pub fn health_v1(state: State<state::HealthState>) -> Json<models::HealthV1> {
    Json(state::health(state.inner()))
}

#[get("/api/v2/health")]
pub fn health_v2(state: State<state::HealthState>) -> Json<models::HealthV2> {
    Json(state::health(state.inner()))
}
