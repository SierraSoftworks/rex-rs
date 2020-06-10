use actix::prelude::*;
use crate::api::APIError;
use std::collections::HashSet;
use super::new_id;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Idea {
    pub id: u128,
    pub collection: u128,
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

json_responder!(IdeaV1 => (req, model) -> req.url_for("get_idea_v1", &vec![model.id.clone().expect("an id to be set")]));

impl From<Idea> for IdeaV1 {
    fn from(idea: Idea) -> Self {
        Self {
            id: Some(format!("{:0>32x}", idea.id)),
            name: idea.name.clone(),
            description: idea.description.clone(),
        }
    }
}

impl Into<Idea> for IdeaV1 {
    fn into(self) -> Idea {
        Idea {
            id: match &self.id {
                Some(id) => u128::from_str_radix(id, 16).unwrap_or_else(|_| new_id()),
                None => new_id(),
            },
            collection: 0,
            name: self.name.clone(),
            description: self.description.clone(),
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

json_responder!(IdeaV2 => (req, model) -> req.url_for("get_idea_v2", &vec![model.id.clone().expect("an id to be set")]));

impl From<Idea> for IdeaV2 {
    fn from(idea: Idea) -> Self {
        Self {
            id: Some(format!("{:0>32x}", idea.id)),
            name: idea.name.clone(),
            description: idea.description.clone(),
            tags: if idea.tags.len() > 0 { Some(idea.tags.clone()) } else { None },
            completed: Some(idea.completed),
        }
    }
}

impl Into<Idea> for IdeaV2 {
    fn into(self) -> Idea {
        Idea {
            id: match &self.id {
                Some(id) => u128::from_str_radix(id, 16).unwrap_or_else(|_| new_id()),
                None => new_id(),
            },
            collection: 0,
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone().unwrap_or_else(|| HashSet::new()),
            completed: self.completed.unwrap_or_else(|| false),
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
    req.url_for("get_collection_idea_v3", &vec![
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
            collection: Some(format!("{:0>32x}", idea.collection)),
            name: idea.name.clone(),
            description: idea.description.clone(),
            tags: if idea.tags.len() > 0 { Some(idea.tags.clone()) } else { None },
            completed: Some(idea.completed),
        }
    }
}

impl Into<Idea> for IdeaV3 {
    fn into(self) -> Idea {
        Idea {
            id: match &self.id {
                Some(id) => u128::from_str_radix(id, 16).unwrap_or_else(|_| new_id()),
                None => new_id(),
            },
            collection: match &self.collection {
                Some(id) => u128::from_str_radix(id, 16).unwrap_or(0),
                None => 0,
            },
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone().unwrap_or_else(|| HashSet::new()),
            completed: self.completed.unwrap_or_else(|| false),
        }
    }
}