#[macro_use]
mod macros;

mod collection;
mod health;
mod idea;
mod role_assignment;
mod user;

use actix::prelude::*;

pub use collection::*;
pub use health::*;
pub use idea::*;
pub use role_assignment::*;
pub use user::*;

pub fn new_id() -> u128 {
    let id = uuid::Uuid::new_v4();
    u128::from_be_bytes(*id.as_bytes())
}

#[derive(Clone)]
pub struct GlobalState {
    pub store: Addr<crate::store::Store>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            store: crate::store::Store::new().start(),
        }
    }
}
