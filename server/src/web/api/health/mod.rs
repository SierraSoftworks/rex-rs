mod get_health;

use actix_web::web;

use crate::services::Services;

pub fn configure<S: Services>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/v1/health")
            .name("get_health_v1")
            .route(web::get().to(get_health::get_health_v1::<S>)),
    )
    .service(
        web::resource("/api/v2/health")
            .name("get_health_v2")
            .route(web::get().to(get_health::get_health_v2::<S>)),
    )
    // An alias, so container probes written against either convention work.
    .service(
        web::resource("/healthz")
            .name("healthz")
            .route(web::get().to(get_health::get_health_v1::<S>)),
    );
}
