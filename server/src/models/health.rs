use rex_api::{HealthV1, HealthV2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Health {
    pub ok: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl From<Health> for HealthV1 {
    fn from(state: Health) -> Self {
        Self { ok: state.ok }
    }
}

impl From<Health> for HealthV2 {
    fn from(state: Health) -> Self {
        Self {
            ok: state.ok,
            started_at: state.started_at,
        }
    }
}
