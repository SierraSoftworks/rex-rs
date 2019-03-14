use super::super::api;

pub fn new_state() -> HealthState {
    HealthState {
        ok: true,
        started_at: chrono::Utc::now(),
    }
}

pub struct HealthState {
    pub ok: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
}


pub fn health<T: api::StateView<HealthState>>(state: &HealthState) -> T {
    T::from_state(state)
}