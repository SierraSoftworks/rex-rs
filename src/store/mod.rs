use super::api;
use actix_web::Result;

mod memory;
mod tablestorage;

#[cfg(any(test, memory_storage, not(any(table_storage))))]
pub type Store = memory::MemoryStore;

#[cfg(table_storage)]
pub type Store = tablestorage::TableStorage;
