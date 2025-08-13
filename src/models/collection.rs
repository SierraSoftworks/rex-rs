use super::new_id;
use crate::api::APIError;
use actix::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub collection_id: u128,
    pub user_id: u128,
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
            id: Some(format!("{:0>32x}", idea.collection_id)),
            user_id: Some(format!("{:0>32x}", idea.user_id)),
            name: idea.name,
        }
    }
}

impl From<CollectionV3> for Collection {
    fn from(val: CollectionV3) -> Self {
        Collection {
            user_id: val
                .user_id
                .clone()
                .and_then(|id| u128::from_str_radix(&id, 16).ok())
                .unwrap_or_default(),
            collection_id: val
                .id
                .clone()
                .and_then(|id| u128::from_str_radix(&id, 16).ok())
                .unwrap_or_else(new_id),
            name: val.name,
        }
    }
}
