use actix::prelude::*;
use crate::api::APIError;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Idea {
    pub id: u128,
    pub collection: u128,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
    pub completed: bool,
}

pub fn new_id() -> u128 {
    let id = Uuid::new_v4();
    u128::from_be_bytes(*id.as_bytes())
}

#[derive(Debug, Default)]
pub struct GetIdea {
    pub id: u128,
    pub collection: u128,
}

impl Message for GetIdea {
    type Result = Result<Idea, APIError>;
}

#[derive(Debug, Default)]
pub struct GetIdeas {
    pub collection: u128,

    pub is_completed: Option<bool>,
    pub tag: Option<String>,
}

impl Message for GetIdeas {
    type Result = Result<Vec<Idea>, APIError>;
}

#[derive(Debug, Default)]
pub struct GetRandomIdea {
    pub collection: u128,

    pub is_completed: Option<bool>,
    pub tag: Option<String>,
}

impl Message for GetRandomIdea {
    type Result = Result<Idea, APIError>;
}

#[derive(Debug, Default)]
pub struct StoreIdea {
    pub id: u128,
    pub collection: u128,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
    pub completed: bool,
}

impl Message for StoreIdea {
    type Result = Result<Idea, APIError>;
}

#[derive(Debug, Default)]
pub struct RemoveIdea {
    pub id: u128,
    pub collection: u128,
}

impl Message for RemoveIdea {
    type Result = Result<(), APIError>;
}
