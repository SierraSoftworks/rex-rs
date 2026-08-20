mod auth;
mod collections;
mod health;
mod ideas;
mod role_assignments;
mod users;
mod utils;

pub use utils::ensure_user_collection;

use actix_web::web;

use crate::services::Services;

pub fn configure<S: Services>(cfg: &mut web::ServiceConfig) {
    health::configure::<S>(cfg);
    auth::configure::<S>(cfg);
    collections::configure::<S>(cfg);
    role_assignments::configure::<S>(cfg);
    ideas::configure::<S>(cfg);
    users::configure::<S>(cfg);
}

/// The path parameters the API's routes use, shared across the resource
/// modules so that a rename cannot drift between a route and its handler.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct IdFilter {
    pub id: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct CollectionFilter {
    pub collection: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct CollectionIdFilter {
    pub collection: String,
    pub id: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct CollectionUserFilter {
    pub collection: String,
    pub user: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct UserFilter {
    pub user: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct QueryFilter {
    pub tag: Option<String>,
    pub complete: Option<bool>,
}

impl From<&QueryFilter> for crate::models::IdeaFilter {
    fn from(query: &QueryFilter) -> Self {
        crate::models::IdeaFilter::new(query.tag.clone(), query.complete)
    }
}
