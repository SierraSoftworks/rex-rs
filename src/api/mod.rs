#[macro_use]
mod macros;

mod auth;
mod collections;
mod error;
mod health;
mod ideas;
mod role_assignments;
mod users;
mod utils;

#[cfg(test)]
pub mod test;

use actix_web::web;

pub use auth::AuthToken;
pub use error::APIError;
pub use utils::ensure_user_collection;

pub fn configure(cfg: &mut web::ServiceConfig) {
    health::configure(cfg);
    collections::configure(cfg);
    role_assignments::configure(cfg);
    ideas::configure(cfg);
    users::configure(cfg);
}
