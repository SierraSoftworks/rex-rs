use super::super::api;
use std::collections::HashSet;
use uuid::Uuid;

use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use http::Method;

#[derive(Debug, Serialize, Deserialize)]
pub struct Idea {
    pub id: u128,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
    pub completed: bool,
}

fn new_id() -> u128 {
    let id = Uuid::new_v4();
    u128::from_be_bytes(*id.as_bytes())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeaV1 {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
}

impl api::StateView<Idea> for IdeaV1 {
    fn to_state(&self) -> Idea {
        Idea {
            id: match &self.id {
                Some(id) => u128::from_str_radix(&id, 16).unwrap_or_else(|_| new_id()),
                None => new_id(),
            },
            name: self.name.clone(),
            description: self.description.clone(),
            tags: HashSet::new(),
            completed: false,
        }
    }

    fn from_state(state: &Idea) -> Self {
        IdeaV1 {
            id: Some(format!("{:x}", state.id)),
            name: state.name.clone(),
            description: state.description.clone(),
        }
    }
}

impl Responder for IdeaV1 {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        if req.method() == Method::POST {
            let location = req.url_for("idea_v1", &vec![self.id.clone().expect("an id to be set")]);

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
    pub tags: HashSet<String>,
    pub completed: Option<bool>,
}

impl api::StateView<Idea> for IdeaV2 {
    fn to_state(&self) -> Idea {
        Idea {
            id: match &self.id {
                Some(id) => u128::from_str_radix(&id, 16).unwrap_or_else(|_| new_id()),
                None => new_id(),
            },
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
            completed: self.completed.unwrap_or_else(|| false),
        }
    }

    fn from_state(state: &Idea) -> Self {
        IdeaV2 {
            id: Some(format!("{:x}", state.id)),
            name: state.name.clone(),
            description: state.description.clone(),
            tags: state.tags.clone(),
            completed: Some(state.completed),
        }
    }
}

impl Responder for IdeaV2 {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        if req.method() == Method::POST {
            let location = req.url_for("idea_v2", &vec![self.id.clone().expect("an id to be set")]);

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
