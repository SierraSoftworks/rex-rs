mod get_collection;
mod get_collections;
mod new_collection;
mod remove_collection;
mod store_collection;

use super::{APIError, AuthToken};
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_collection::get_collection_v3)
        .service(get_collections::get_collections_v3)
        .service(new_collection::new_collection_v3)
        .service(store_collection::store_collection_v3)
        .service(remove_collection::remove_collection_v3);
}

#[derive(Debug, Deserialize, Serialize)]
struct CollectionFilter {
    collection: String,
}
