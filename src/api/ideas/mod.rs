
use actix_web::web;
use super::{AuthToken, APIError, ensure_user_collection};

mod new_idea;
mod get_ideas;
mod get_idea;
mod get_random_idea;
mod store_idea;
mod remove_idea;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_ideas::get_ideas_v1)
        .service(get_random_idea::get_random_idea_v1)
        .service(get_idea::get_idea_v1)
        .service(new_idea::new_idea_v1)
        .service(store_idea::store_idea_v1)
        .service(remove_idea::remove_idea_v1);

    cfg.service(get_ideas::get_ideas_v2)
        .service(get_random_idea::get_random_idea_v2)
        .service(get_idea::get_idea_v2)
        .service(new_idea::new_idea_v2)
        .service(store_idea::store_idea_v2)
        .service(remove_idea::remove_idea_v2);

    cfg.service(get_ideas::get_ideas_v3)
        .service(get_ideas::get_collection_ideas_v3)
        .service(get_random_idea::get_random_idea_v3)
        .service(get_random_idea::get_random_collection_idea_v3)
        .service(get_idea::get_idea_v3)
        .service(get_idea::get_collection_idea_v3)
        .service(new_idea::new_idea_v3)
        .service(new_idea::new_collection_idea_v3)
        .service(store_idea::store_idea_v3)
        .service(store_idea::store_collection_idea_v3)
        .service(remove_idea::remove_idea_v3)
        .service(remove_idea::remove_collection_idea_v3);
}

#[derive(Deserialize, Serialize)]
struct IdFilter {
    id: String,
}

#[derive(Deserialize, Serialize)]
struct CollectionFilter {
    collection: String,
}

#[derive(Deserialize, Serialize)]
struct CollectionIdFilter {
    collection: String,
    id: String,
}

#[derive(Deserialize)]
pub struct QueryFilter {
    tag: Option<String>,
    complete: Option<bool>,
}
