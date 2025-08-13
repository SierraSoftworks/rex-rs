use super::new_id;
use crate::api::APIError;
use actix::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Idea {
    pub id: u128,
    pub collection_id: u128,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
    pub completed: bool,
}

actor_message!(GetIdea(id: u128, collection: u128) -> Idea);

actor_message!(GetIdeas(collection: u128, tag: Option<String>, is_completed: Option<bool>) -> Vec<Idea>);

actor_message!(GetRandomIdea(collection: u128, tag: Option<String>, is_completed: Option<bool>) -> Idea);

actor_message!(StoreIdea(id: u128, collection: u128, name: String, description: String, tags: HashSet<String>, completed: bool) -> Idea);

actor_message!(RemoveIdea(id: u128, collection: u128) -> ());

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeaV1 {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
}

json_responder!(IdeaV1 => (req, model) -> req.url_for("get_idea_v1", vec![model.id.clone().expect("an id to be set")]));

impl From<Idea> for IdeaV1 {
    fn from(idea: Idea) -> Self {
        Self {
            id: Some(format!("{:0>32x}", idea.id)),
            name: idea.name.clone(),
            description: idea.description,
        }
    }
}

impl From<IdeaV1> for Idea {
    fn from(val: IdeaV1) -> Self {
        Idea {
            id: val
                .id
                .clone()
                .and_then(|id| u128::from_str_radix(&id, 16).ok())
                .unwrap_or_default(),
            collection_id: 0,
            name: val.name.clone(),
            description: val.description,
            tags: HashSet::new(),
            completed: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeaV2 {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub tags: Option<HashSet<String>>,
    pub completed: Option<bool>,
}

json_responder!(IdeaV2 => (req, model) -> req.url_for("get_idea_v2", vec![model.id.clone().expect("an id to be set")]));

impl From<Idea> for IdeaV2 {
    fn from(idea: Idea) -> Self {
        Self {
            id: Some(format!("{:0>32x}", idea.id)),
            name: idea.name.clone(),
            description: idea.description.clone(),
            tags: if !idea.tags.is_empty() {
                Some(idea.tags.clone())
            } else {
                None
            },
            completed: Some(idea.completed),
        }
    }
}

impl From<IdeaV2> for Idea {
    fn from(val: IdeaV2) -> Self {
        Idea {
            id: val
                .id
                .clone()
                .and_then(|id| u128::from_str_radix(&id, 16).ok())
                .unwrap_or_default(),
            collection_id: 0,
            name: val.name.clone(),
            description: val.description.clone(),
            tags: val.tags.clone().unwrap_or_default(),
            completed: val.completed.unwrap_or(false),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeaV3 {
    pub collection: Option<String>,
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub tags: Option<HashSet<String>>,
    pub completed: Option<bool>,
}

json_responder!(IdeaV3 => (req, model) -> if req.uri().path().contains("/collection/") {
    req.url_for("get_collection_idea_v3", vec![
        model.collection.clone().expect("a collection id"),
        model.id.clone().expect("an idea id")
    ])
} else {
    req.url_for("get_idea_v3", vec![model.id.clone().expect("an idea id")])
});

impl From<Idea> for IdeaV3 {
    fn from(idea: Idea) -> Self {
        Self {
            id: Some(format!("{:0>32x}", idea.id)),
            collection: Some(format!("{:0>32x}", idea.collection_id)),
            name: idea.name.clone(),
            description: idea.description.clone(),
            tags: if !idea.tags.is_empty() {
                Some(idea.tags.clone())
            } else {
                None
            },
            completed: Some(idea.completed),
        }
    }
}

impl From<IdeaV3> for Idea {
    fn from(val: IdeaV3) -> Self {
        Idea {
            id: val
                .id
                .clone()
                .and_then(|id| u128::from_str_radix(&id, 16).ok())
                .unwrap_or_else(new_id),
            collection_id: val
                .collection
                .clone()
                .and_then(|id| u128::from_str_radix(&id, 16).ok())
                .unwrap_or_default(),
            name: val.name.clone(),
            description: val.description.clone(),
            tags: val.tags.clone().unwrap_or_default(),
            completed: val.completed.unwrap_or(false),
        }
    }
}
