mod get_user;

use super::{APIError, AuthToken};
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user::get_user_v3);
}

#[derive(Debug, Deserialize, Serialize)]
struct UserFilter {
    user: String,
}
