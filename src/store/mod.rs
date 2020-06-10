#[cfg(any(test, not(feature = "table_storage")))]
mod memory;

#[cfg(any(test, not(feature = "table_storage")))]
pub type Store = memory::MemoryStore;

#[cfg(all(not(test), feature = "table_storage"))]
mod tablestorage;

#[cfg(all(not(test), feature = "table_storage"))]
pub type Store = tablestorage::TableStorage;
