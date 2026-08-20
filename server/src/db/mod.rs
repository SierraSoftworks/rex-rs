//! Storage for Rex.
//!
//! Two implementations of [`Store`] ship: [`SqliteStore`], which is what
//! production runs on, and [`MemoryStore`], which backs the handler tests and
//! the zero-setup `storage = "memory"` dev mode. The [`conformance`] suite runs
//! the same assertions against both so the thing under test stays the thing
//! that ships.

mod memory;
mod migrations;
mod sqlite;

#[cfg(test)]
pub mod conformance;

pub use memory::MemoryStore;
pub use migrations::MIGRATIONS;
pub use sqlite::SqliteStore;

use rex_api::ApiError;

use crate::models::*;

/// The storage operations Rex needs, mirroring the sixteen actor messages the
/// service used before the store became a plain trait.
///
/// The trait is deliberately *not* object safe (`async fn` in traits never is):
/// everything downstream stays generic over `S: Store`, so there is never a
/// `dyn Store` to want.
#[allow(async_fn_in_trait)]
pub trait Store: Clone + Send + Sync + 'static {
    async fn get_idea(&self, collection: Id, id: Id) -> Result<Idea, ApiError>;
    async fn get_ideas(&self, collection: Id, filter: IdeaFilter) -> Result<Vec<Idea>, ApiError>;
    async fn get_random_idea(&self, collection: Id, filter: IdeaFilter) -> Result<Idea, ApiError>;
    async fn store_idea(&self, idea: Idea) -> Result<Idea, ApiError>;
    async fn remove_idea(&self, collection: Id, id: Id) -> Result<(), ApiError>;

    async fn get_collection(&self, id: Id, principal: Id) -> Result<Collection, ApiError>;
    async fn get_collections(&self, principal: Id) -> Result<Vec<Collection>, ApiError>;
    async fn store_collection(&self, collection: Collection) -> Result<Collection, ApiError>;
    async fn remove_collection(&self, id: Id, principal: Id) -> Result<(), ApiError>;

    async fn get_role_assignment(
        &self,
        collection: Id,
        principal: Id,
    ) -> Result<RoleAssignment, ApiError>;
    async fn get_role_assignments(&self, collection: Id) -> Result<Vec<RoleAssignment>, ApiError>;
    async fn store_role_assignment(
        &self,
        assignment: RoleAssignment,
    ) -> Result<RoleAssignment, ApiError>;
    async fn remove_role_assignment(&self, collection: Id, principal: Id) -> Result<(), ApiError>;

    async fn get_user(&self, email_hash: Id) -> Result<User, ApiError>;
    async fn store_user(&self, user: User) -> Result<User, ApiError>;

    async fn health(&self) -> Result<Health, ApiError>;
}

// ── Shared error text ────────────────────────────────────────────────────────
//
// Both stores return the same messages for the same situations; the conformance
// suite asserts on the status codes, and the UI shows the messages verbatim.

pub(crate) fn collection_not_found() -> ApiError {
    ApiError::not_found(
        "The collection ID you provided could not be found. Please check it and try again.",
    )
}

pub(crate) fn idea_not_found() -> ApiError {
    ApiError::not_found(
        "The idea ID you provided could not be found. Please check it and try again.",
    )
}

pub(crate) fn no_random_idea() -> ApiError {
    ApiError::not_found("No random ideas were available.")
}

pub(crate) fn principal_not_found() -> ApiError {
    ApiError::not_found(
        "The principal ID you provided could not be found. This likely means that you do not yet have any collections.",
    )
}

pub(crate) fn user_not_found() -> ApiError {
    ApiError::not_found(
        "No user could be found with the email hash you provided. Please check it and try again.",
    )
}

/// Returned when the caller holds no role on a collection.
///
/// A 403 rather than a 404 here matches what the service has always sent, and
/// the UI's permission messaging depends on it.
pub(crate) fn no_access() -> ApiError {
    ApiError::forbidden("You do not have permission to access this resource.")
}
