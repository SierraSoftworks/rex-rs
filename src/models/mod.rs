mod collection;
mod idea;
mod role_assignment;
mod health;
mod stateview;

use actix::prelude::*;

pub use collection::*;
pub use health::*;
pub use idea::*;
pub use role_assignment::*;
pub use stateview::*;

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