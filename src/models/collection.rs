use actix::prelude::*;
use crate::api::APIError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: u128,
    pub principal_id: u128,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct GetCollection {
    pub id: u128,
    pub principal_id: u128,
}

impl Message for GetCollection {
    type Result = Result<Collection, APIError>;
}

#[derive(Debug, Default)]
pub struct GetCollections {
    pub principal_id: u128,
}

impl Message for GetCollections {
    type Result = Result<Vec<Collection>, APIError>;
}

#[derive(Debug, Default)]
pub struct StoreCollection {
    pub collection_id: u128,
    pub principal_id: u128,
    pub name: String,
}

impl Message for StoreCollection {
    type Result = Result<Collection, APIError>;
}

#[derive(Debug, Default)]
pub struct RemoveCollection {
    pub id: u128,
    pub principal_id: u128,
}

impl Message for RemoveCollection {
    type Result = Result<(), APIError>;
}
