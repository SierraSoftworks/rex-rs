mod get_role_assignment;
mod get_role_assignments;
mod remove_role_assignment;
mod store_role_assignment;

use actix_web::web;

use crate::services::Services;

pub fn configure<S: Services>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/v3/collection/{collection}/users")
            .name("get_role_assignments_v3")
            .route(web::get().to(get_role_assignments::get_role_assignments_v3::<S>)),
    )
    .service(
        web::resource("/api/v3/collection/{collection}/user/{user}")
            .name("get_role_assignment_v3")
            .route(web::get().to(get_role_assignment::get_role_assignment_v3::<S>))
            .route(web::put().to(store_role_assignment::store_role_assignment_v3::<S>))
            .route(web::delete().to(remove_role_assignment::remove_role_assignment_v3::<S>)),
    );
}
