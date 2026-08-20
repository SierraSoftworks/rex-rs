mod get_idea;
mod get_ideas;
mod get_random_idea;
mod new_idea;
mod remove_idea;
mod store_idea;

use actix_web::web;

use crate::services::Services;

pub fn configure<S: Services>(cfg: &mut web::ServiceConfig) {
    // `/idea/random` has to be registered ahead of `/idea/{id}`, or the literal
    // gets swallowed by the parameter.
    cfg.service(
        web::resource("/api/v1/ideas")
            .name("get_ideas_v1")
            .route(web::get().to(get_ideas::get_ideas_v1::<S>))
            .route(web::post().to(new_idea::new_idea_v1::<S>)),
    )
    .service(
        web::resource("/api/v1/idea/random")
            .name("get_random_idea_v1")
            .route(web::get().to(get_random_idea::get_random_idea_v1::<S>)),
    )
    .service(
        web::resource("/api/v1/idea/{id}")
            .name("get_idea_v1")
            .route(web::get().to(get_idea::get_idea_v1::<S>))
            .route(web::put().to(store_idea::store_idea_v1::<S>))
            .route(web::delete().to(remove_idea::remove_idea_v1::<S>)),
    );

    cfg.service(
        web::resource("/api/v2/ideas")
            .name("get_ideas_v2")
            .route(web::get().to(get_ideas::get_ideas_v2::<S>))
            .route(web::post().to(new_idea::new_idea_v2::<S>)),
    )
    .service(
        web::resource("/api/v2/idea/random")
            .name("get_random_idea_v2")
            .route(web::get().to(get_random_idea::get_random_idea_v2::<S>)),
    )
    .service(
        web::resource("/api/v2/idea/{id}")
            .name("get_idea_v2")
            .route(web::get().to(get_idea::get_idea_v2::<S>))
            .route(web::put().to(store_idea::store_idea_v2::<S>))
            .route(web::delete().to(remove_idea::remove_idea_v2::<S>)),
    );

    cfg.service(
        web::resource("/api/v3/ideas")
            .name("get_ideas_v3")
            .route(web::get().to(get_ideas::get_ideas_v3::<S>))
            .route(web::post().to(new_idea::new_idea_v3::<S>)),
    )
    .service(
        web::resource("/api/v3/idea/random")
            .name("get_random_idea_v3")
            .route(web::get().to(get_random_idea::get_random_idea_v3::<S>)),
    )
    .service(
        web::resource("/api/v3/idea/{id}")
            .name("get_idea_v3")
            .route(web::get().to(get_idea::get_idea_v3::<S>))
            .route(web::put().to(store_idea::store_idea_v3::<S>))
            .route(web::delete().to(remove_idea::remove_idea_v3::<S>)),
    )
    .service(
        web::resource("/api/v3/collection/{collection}/ideas")
            .name("get_collection_ideas_v3")
            .route(web::get().to(get_ideas::get_collection_ideas_v3::<S>))
            .route(web::post().to(new_idea::new_collection_idea_v3::<S>)),
    )
    .service(
        web::resource("/api/v3/collection/{collection}/idea/random")
            .name("get_random_collection_idea_v3")
            .route(web::get().to(get_random_idea::get_random_collection_idea_v3::<S>)),
    )
    .service(
        web::resource("/api/v3/collection/{collection}/idea/{id}")
            .name("get_collection_idea_v3")
            .route(web::get().to(get_idea::get_collection_idea_v3::<S>))
            .route(web::put().to(store_idea::store_collection_idea_v3::<S>))
            .route(web::delete().to(remove_idea::remove_collection_idea_v3::<S>)),
    );
}
