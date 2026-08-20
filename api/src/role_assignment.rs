use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RoleAssignmentV3 {
    #[serde(rename = "collectionId")]
    pub collection_id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub role: String,
}
