use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use rand::seq::IteratorRandom;
use rex_api::ApiError;
use tracing::instrument;

use super::{
    Store, collection_not_found, idea_not_found, no_access, no_random_idea, principal_not_found,
    user_not_found,
};
use crate::models::*;

/// An entirely in-memory [`Store`].
///
/// This is what every handler test runs against — no SQLite in the loop, no
/// files, no cleanup — and it doubles as the `storage = "memory"` dev mode, so
/// it stays honest rather than rotting behind `#[cfg(test)]`.
#[derive(Clone)]
pub struct MemoryStore {
    started_at: chrono::DateTime<chrono::Utc>,
    ideas: Arc<RwLock<BTreeMap<Id, BTreeMap<Id, Idea>>>>,
    collections: Arc<RwLock<BTreeMap<Id, String>>>,
    role_assignments: Arc<RwLock<BTreeMap<Id, BTreeMap<Id, RoleAssignment>>>>,
    users: Arc<RwLock<BTreeMap<Id, User>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            started_at: chrono::Utc::now(),
            ideas: Default::default(),
            collections: Default::default(),
            role_assignments: Default::default(),
            users: Default::default(),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

/// A poisoned lock means another thread panicked mid-write; there is nothing
/// the caller can do about it, so it becomes a plain 500.
fn poisoned() -> ApiError {
    ApiError::new(
        500,
        "Internal Server Error",
        "The service is currently unavailable, please try again later.",
    )
}

macro_rules! read {
    ($self:ident . $field:ident) => {
        $self.$field.read().map_err(|_| poisoned())?
    };
}

macro_rules! write {
    ($self:ident . $field:ident) => {
        $self.$field.write().map_err(|_| poisoned())?
    };
}

impl Store for MemoryStore {
    #[instrument(name = "store.get_idea", skip(self), err, fields(db.system = "memory", db.operation = "get_idea"))]
    async fn get_idea(&self, collection: Id, id: Id) -> Result<Idea, ApiError> {
        if !read!(self.collections).contains_key(&collection) {
            return Err(collection_not_found());
        }

        read!(self.ideas)
            .get(&collection)
            .and_then(|ideas| ideas.get(&id))
            .cloned()
            .ok_or_else(idea_not_found)
    }

    #[instrument(name = "store.get_ideas", skip(self), err, fields(db.system = "memory", db.operation = "get_ideas"))]
    async fn get_ideas(&self, collection: Id, filter: IdeaFilter) -> Result<Vec<Idea>, ApiError> {
        if !read!(self.collections).contains_key(&collection) {
            return Err(collection_not_found());
        }

        Ok(read!(self.ideas)
            .get(&collection)
            .map(|ideas| {
                ideas
                    .values()
                    .filter(|idea| filter.matches(idea))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    #[instrument(name = "store.get_random_idea", skip(self), err, fields(db.system = "memory", db.operation = "get_random_idea"))]
    async fn get_random_idea(&self, collection: Id, filter: IdeaFilter) -> Result<Idea, ApiError> {
        if !read!(self.collections).contains_key(&collection) {
            return Err(collection_not_found());
        }

        read!(self.ideas)
            .get(&collection)
            .and_then(|ideas| {
                ideas
                    .values()
                    .filter(|idea| filter.matches(idea))
                    .choose(&mut rand::rng())
                    .cloned()
            })
            .ok_or_else(no_random_idea)
    }

    #[instrument(name = "store.store_idea", skip(self), err, fields(db.system = "memory", db.operation = "store_idea"))]
    async fn store_idea(&self, idea: Idea) -> Result<Idea, ApiError> {
        if !read!(self.collections).contains_key(&idea.collection_id) {
            return Err(collection_not_found());
        }

        write!(self.ideas)
            .entry(idea.collection_id)
            .or_default()
            .insert(idea.id, idea.clone());

        Ok(idea)
    }

    #[instrument(name = "store.remove_idea", skip(self), err, fields(db.system = "memory", db.operation = "remove_idea"))]
    async fn remove_idea(&self, collection: Id, id: Id) -> Result<(), ApiError> {
        if !read!(self.collections).contains_key(&collection) {
            return Err(collection_not_found());
        }

        write!(self.ideas)
            .get_mut(&collection)
            .and_then(|ideas| ideas.remove(&id))
            .map(|_| ())
            .ok_or_else(idea_not_found)
    }

    #[instrument(name = "store.get_collection", skip(self), err, fields(db.system = "memory", db.operation = "get_collection"))]
    async fn get_collection(&self, id: Id, principal: Id) -> Result<Collection, ApiError> {
        let has_access = read!(self.role_assignments)
            .get(&id)
            .is_some_and(|assignments| assignments.contains_key(&principal));

        if !has_access {
            return Err(collection_not_found());
        }

        read!(self.collections)
            .get(&id)
            .map(|name| Collection {
                collection_id: id,
                user_id: principal,
                name: name.clone(),
            })
            .ok_or_else(collection_not_found)
    }

    #[instrument(name = "store.get_collections", skip(self), err, fields(db.system = "memory", db.operation = "get_collections"))]
    async fn get_collections(&self, principal: Id) -> Result<Vec<Collection>, ApiError> {
        let collections = read!(self.collections);

        let mine: Vec<Collection> = read!(self.role_assignments)
            .iter()
            .filter(|(_, assignments)| assignments.contains_key(&principal))
            .filter_map(|(id, _)| {
                collections.get(id).map(|name| Collection {
                    collection_id: *id,
                    user_id: principal,
                    name: name.clone(),
                })
            })
            .collect();

        if mine.is_empty() {
            // Preserved from the Table Storage implementation, where an empty
            // partition was indistinguishable from a missing one.
            return Err(principal_not_found());
        }

        Ok(mine)
    }

    #[instrument(name = "store.store_collection", skip(self), err, fields(db.system = "memory", db.operation = "store_collection"))]
    async fn store_collection(&self, collection: Collection) -> Result<Collection, ApiError> {
        write!(self.collections).insert(collection.collection_id, collection.name.clone());

        Ok(collection)
    }

    #[instrument(name = "store.remove_collection", skip(self), err, fields(db.system = "memory", db.operation = "remove_collection"))]
    async fn remove_collection(&self, id: Id, principal: Id) -> Result<(), ApiError> {
        let role = read!(self.role_assignments)
            .get(&id)
            .and_then(|assignments| assignments.get(&principal))
            .map(|assignment| assignment.role)
            .ok_or_else(collection_not_found)?;

        if role == Role::Owner {
            // Owners delete the collection outright. Under the old Table
            // Storage layout each member held their own copy, so an owner's
            // delete left everyone else's copy (and every idea) orphaned but
            // still reachable; normalising the schema fixes that.
            write!(self.collections).remove(&id);
            write!(self.ideas).remove(&id);
            write!(self.role_assignments).remove(&id);
        } else {
            // Everyone else is just leaving the collection.
            write!(self.role_assignments)
                .get_mut(&id)
                .and_then(|assignments| assignments.remove(&principal));
        }

        Ok(())
    }

    #[instrument(name = "store.get_role_assignment", skip(self), err, fields(db.system = "memory", db.operation = "get_role_assignment"))]
    async fn get_role_assignment(
        &self,
        collection: Id,
        principal: Id,
    ) -> Result<RoleAssignment, ApiError> {
        read!(self.role_assignments)
            .get(&collection)
            .and_then(|assignments| assignments.get(&principal))
            .cloned()
            .ok_or_else(no_access)
    }

    #[instrument(name = "store.get_role_assignments", skip(self), err, fields(db.system = "memory", db.operation = "get_role_assignments"))]
    async fn get_role_assignments(&self, collection: Id) -> Result<Vec<RoleAssignment>, ApiError> {
        if !read!(self.collections).contains_key(&collection) {
            return Err(collection_not_found());
        }

        Ok(read!(self.role_assignments)
            .get(&collection)
            .map(|assignments| assignments.values().cloned().collect())
            .unwrap_or_default())
    }

    #[instrument(name = "store.store_role_assignment", skip(self), err, fields(db.system = "memory", db.operation = "store_role_assignment"))]
    async fn store_role_assignment(
        &self,
        assignment: RoleAssignment,
    ) -> Result<RoleAssignment, ApiError> {
        if !read!(self.collections).contains_key(&assignment.collection_id) {
            return Err(collection_not_found());
        }

        write!(self.role_assignments)
            .entry(assignment.collection_id)
            .or_default()
            .insert(assignment.user_id, assignment.clone());

        Ok(assignment)
    }

    #[instrument(name = "store.remove_role_assignment", skip(self), err, fields(db.system = "memory", db.operation = "remove_role_assignment"))]
    async fn remove_role_assignment(&self, collection: Id, principal: Id) -> Result<(), ApiError> {
        if !read!(self.collections).contains_key(&collection) {
            return Err(collection_not_found());
        }

        write!(self.role_assignments)
            .get_mut(&collection)
            .and_then(|assignments| assignments.remove(&principal))
            .map(|_| ())
            .ok_or_else(principal_not_found)
    }

    #[instrument(name = "store.get_user", skip(self), err, fields(db.system = "memory", db.operation = "get_user"))]
    async fn get_user(&self, email_hash: Id) -> Result<User, ApiError> {
        read!(self.users)
            .get(&email_hash)
            .cloned()
            .ok_or_else(user_not_found)
    }

    #[instrument(name = "store.store_user", skip(self), err, fields(db.system = "memory", db.operation = "store_user"))]
    async fn store_user(&self, user: User) -> Result<User, ApiError> {
        write!(self.users).insert(user.email_hash, user.clone());

        Ok(user)
    }

    #[instrument(name = "store.health", skip(self), err, fields(db.system = "memory", db.operation = "health"))]
    async fn health(&self) -> Result<Health, ApiError> {
        Ok(Health {
            ok: true,
            started_at: self.started_at,
        })
    }
}
