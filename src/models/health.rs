use actix::prelude::*;
use crate::api::APIError;

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

impl Into<Health> for HealthV1 {
    fn into(self) -> Health {
        Health {
            ok: self.ok,
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

impl Into<Health> for HealthV2 {
    fn into(self) -> Health {
        Health {
            ok: self.ok,
            started_at: self.started_at,
        }
    }
}