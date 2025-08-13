mod get_role_assignment;
mod get_role_assignments;
mod remove_role_assignment;
mod store_role_assignment;

use super::{APIError, AuthToken};
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_role_assignment::get_role_assignment_v3)
        .service(get_role_assignments::get_role_assignments_v3)
        .service(store_role_assignment::store_role_assignment_v3)
        .service(remove_role_assignment::remove_role_assignment_v3);
}

#[derive(Debug, Deserialize, Serialize)]
struct CollectionFilter {
    collection: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CollectionUserFilter {
    collection: String,
    user: String,
}
