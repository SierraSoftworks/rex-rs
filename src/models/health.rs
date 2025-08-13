use crate::api::APIError;
use actix::prelude::*;

#[derive(Clone, Copy)]
pub struct Health {
    pub ok: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

actor_message!(GetHealth() -> Health);

#[derive(Serialize, Deserialize)]
pub struct HealthV1 {
    pub ok: bool,
}

json_responder!(HealthV1);

impl From<Health> for HealthV1 {
    fn from(state: Health) -> Self {
        Self { ok: state.ok }
    }
}

impl From<HealthV1> for Health {
    fn from(val: HealthV1) -> Self {
        Health {
            ok: val.ok,
            started_at: chrono::Utc::now(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HealthV2 {
    pub ok: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

json_responder!(HealthV2);

impl From<Health> for HealthV2 {
    fn from(state: Health) -> Self {
        Self {
            ok: state.ok,
            started_at: state.started_at,
        }
    }
}

impl From<HealthV2> for Health {
    fn from(val: HealthV2) -> Self {
        Health {
            ok: val.ok,
            started_at: val.started_at,
        }
    }
}
