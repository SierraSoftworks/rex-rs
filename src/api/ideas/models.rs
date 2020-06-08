use crate::models::*;
use std::collections::HashSet;

use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use http::Method;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeaV1 {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
}

impl StateView<Idea> for IdeaV1 {
    fn to_state(&self) -> Idea {
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

    fn from_state(state: &Idea) -> Self {
        IdeaV1 {
            id: Some(format!("{:0>32x}", state.id)),
            name: state.name.clone(),
            description: state.description.clone(),
        }
    }
}

impl From<Idea> for IdeaV1 {
    fn from(idea: Idea) -> Self {
        Self::from_state(&idea)
    }
}

impl Into<Idea> for IdeaV1 {
    fn into(self) -> Idea {
        self.to_state()
    }
}

impl Responder for IdeaV1 {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        if req.method() == Method::POST {
            let location = req.url_for("get_idea_v1", &vec![self.id.clone().expect("an id to be set")]);

            ready(Ok(HttpResponse::Created()
                .content_type("application/json")
                .header(
                    "Location",
                    location.expect("a location string").into_string(),
                )
                .json(&self)))
        } else {
            ready(Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(&self)))
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

impl StateView<Idea> for IdeaV2 {
    fn to_state(&self) -> Idea {
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

    fn from_state(state: &Idea) -> Self {
        IdeaV2 {
            id: Some(format!("{:0>32x}", state.id)),
            name: state.name.clone(),
            description: state.description.clone(),
            tags: if state.tags.len() > 0 { Some(state.tags.clone()) } else { None },
            completed: Some(state.completed),
        }
    }
}

impl From<Idea> for IdeaV2 {
    fn from(idea: Idea) -> Self {
        Self::from_state(&idea)
    }
}

impl Into<Idea> for IdeaV2 {
    fn into(self) -> Idea {
        self.to_state()
    }
}

impl Responder for IdeaV2 {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        if req.method() == Method::POST {
            let location = req.url_for("get_idea_v2", &vec![self.id.clone().expect("an id to be set")]);

            ready(Ok(HttpResponse::Created()
                .content_type("application/json")
                .header(
                    "Location",
                    location.expect("a location string").into_string(),
                )
                .json(&self)))
        } else {
            ready(Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(&self)))
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

impl StateView<Idea> for IdeaV3 {
    fn to_state(&self) -> Idea {
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

    fn from_state(state: &Idea) -> Self {
        IdeaV3 {
            id: Some(format!("{:0>32x}", state.id)),
            collection: Some(format!("{:0>32x}", state.collection)),
            name: state.name.clone(),
            description: state.description.clone(),
            tags: if state.tags.len() > 0 { Some(state.tags.clone()) } else { None },
            completed: Some(state.completed),
        }
    }
}

impl From<Idea> for IdeaV3 {
    fn from(idea: Idea) -> Self {
        Self::from_state(&idea)
    }
}

impl Into<Idea> for IdeaV3 {
    fn into(self) -> Idea {
        self.to_state()
    }
}

impl Responder for IdeaV3 {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        if req.method() == Method::POST {
            let location = if req.uri().path().contains("/collection/") {
                req.url_for("get_collection_idea_v3", &vec![
                    self.collection.clone().expect("a collection id"),
                    self.id.clone().expect("an idea id")
                ]) 
            } else { req.url_for("get_idea_v3", vec![self.id.clone().expect("an idea id")]) };

            ready(Ok(HttpResponse::Created()
                .content_type("application/json")
                .header(
                    "Location",
                    location.expect("a location string").into_string(),
                )
                .json(&self)))
        } else {
            ready(Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(&self)))
        }
    }
}
