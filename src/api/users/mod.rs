mod get_user;

use actix_web::web;
use super::{AuthToken, APIError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg 
        .service(get_user::get_user_v3);
}

#[derive(Deserialize, Serialize)]
struct UserFilter {
    user: String,
}