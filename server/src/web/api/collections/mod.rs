mod get_collection;
mod get_collections;
mod new_collection;
mod remove_collection;
mod store_collection;

use actix_web::web;

use crate::services::Services;

pub fn configure<S: Services>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/v3/collections")
            .name("get_collections_v3")
            .route(web::get().to(get_collections::get_collections_v3::<S>))
            .route(web::post().to(new_collection::new_collection_v3::<S>)),
    )
    .service(
        web::resource("/api/v3/collection/{collection}")
            .name("get_collection_v3")
            .route(web::get().to(get_collection::get_collection_v3::<S>))
            .route(web::put().to(store_collection::store_collection_v3::<S>))
            .route(web::delete().to(remove_collection::remove_collection_v3::<S>)),
    );
}
