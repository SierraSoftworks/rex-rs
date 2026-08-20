use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CollectionV3 {
    pub id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub name: String,
}
