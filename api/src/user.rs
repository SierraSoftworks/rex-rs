use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct UserV3 {
    pub id: String,

    #[serde(rename = "emailHash")]
    pub email_hash: String,

    #[serde(rename = "firstName")]
    pub first_name: String,
}
