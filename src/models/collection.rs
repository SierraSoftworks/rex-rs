use actix::prelude::*;
use crate::api::APIError;
use super::new_id;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: u128,
    pub principal_id: u128,
    pub name: String,
}

actor_message!(GetCollection(id: u128, principal_id: u128) -> Collection);

actor_message!(GetCollections(principal_id: u128) -> Vec<Collection>);

actor_message!(StoreCollection(collection_id: u128, principal_id: u128, name: String) -> Collection);

actor_message!(RemoveCollection(id: u128, principal_id: u128) -> ());

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionV3 {
    pub id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub name: String,
}

json_responder!(CollectionV3 => (req, model) -> req.url_for("get_collection_v3", vec![model.id.clone().expect("a collection id")]));

impl From<Collection> for CollectionV3 {
    fn from(idea: Collection) -> Self {
        Self {
            id: Some(format!("{:0>32x}", idea.id)),
            user_id: Some(format!("{:0>32x}", idea.principal_id)),
            name: idea.name.clone(),
        }
    }
}

impl Into<Collection> for CollectionV3 {
    fn into(self) -> Collection {
        Collection {
            principal_id: match &self.user_id {
                Some(id) => u128::from_str_radix(&id, 16).unwrap_or_else(|_| new_id()),
                None => new_id(),
            },
            id: match &self.id {
                Some(id) => u128::from_str_radix(&id, 16).unwrap_or(0),
                None => 0,
            },
            name: self.name.clone(),
        }
    }
}