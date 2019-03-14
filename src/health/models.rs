use super::super::api;
use super::state;

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
