use actix_web::web;

mod get_health;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_health::get_health_v1)
        .service(get_health::get_health_v2);
}
