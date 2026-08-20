use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthV1 {
    pub ok: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthV2 {
    pub ok: bool,
    pub started_at: chrono::DateTime<chrono::Utc>,
}
