mod get_user;

use actix_web::web;

use crate::services::Services;

pub fn configure<S: Services>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/v3/user/{user}")
            .name("get_user_v3")
            .route(web::get().to(get_user::get_user_v3::<S>)),
    );
}
