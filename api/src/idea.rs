use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeaV1 {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeaV2 {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub tags: Option<HashSet<String>>,
    pub completed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IdeaV3 {
    pub collection: Option<String>,
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub tags: Option<HashSet<String>>,
    pub completed: Option<bool>,
}
