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
    #[cfg(any(test, memory_storage, not(any(table_storage))))]
    pub fn new() -> Self {
        Self {
            store: crate::store::Store::new().start(),
        }
    }
    
    #[cfg(table_storage)]
    pub fn new() -> Self {
        let connection_str = std::env::var("TABLE_STORAGE_CONNECTION_STRING").expect("Set the TABLE_STORAGE_CONNECTION_STRING environment variable before starting the server.");

        Self {
            store: crate::store::Store::new(connection_str.as_str()).start(),
        }
    }
}