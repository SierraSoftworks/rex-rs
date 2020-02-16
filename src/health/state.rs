use super::super::api;

#[derive(Clone, Copy)]
pub struct HealthState {
    pub ok: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl HealthState {
    pub fn new() -> Self {
        Self {
            ok: true,
            started_at: chrono::Utc::now(),
        }
    }

    pub fn health<T: api::StateView<HealthState>>(&self) -> T {
        T::from_state(self)
    }
}
