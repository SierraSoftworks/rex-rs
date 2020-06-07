use actix::prelude::*;
use crate::api::APIError;

#[derive(Clone, Copy)]
pub struct Health {
    pub ok: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetHealth {
    
}

impl Message for GetHealth {
    type Result = Result<Health, APIError>;
}