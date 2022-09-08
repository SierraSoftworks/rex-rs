use std::{collections::BTreeMap, sync::Arc};
use crate::{models::*, trace_handler};
use crate::api::APIError;
use std::sync::RwLock;
use rand::seq::IteratorRandom;
use actix::prelude::*;

pub struct MemoryStore {
    started_at: chrono::DateTime<chrono::Utc>,
    ideas: Arc<RwLock<BTreeMap<u128, BTreeMap<u128, Idea>>>>,
    collections: Arc<RwLock<BTreeMap<u128, BTreeMap<u128, Collection>>>>,
    role_assignments: Arc<RwLock<BTreeMap<u128, BTreeMap<u128, RoleAssignment>>>>,
    users: Arc<RwLock<BTreeMap<u128, User>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            started_at: chrono::Utc::now(),
            ideas: Arc::new(RwLock::new(BTreeMap::new())),
            collections: Arc::new(RwLock::new(BTreeMap::new())),
            role_assignments: Arc::new(RwLock::new(BTreeMap::new())),
            users: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl Actor for MemoryStore {
    type Context = Context<Self>;
}

trace_handler!(MemoryStore, GetHealth, Result<Health, APIError>);

impl Handler<GetHealth> for MemoryStore {
    type Result = Result<Health, APIError>;

    fn handle(&mut self, _: GetHealth, _: &mut Self::Context) -> Self::Result {
        Ok(Health {
            ok: true,
            started_at: self.started_at,
        })
    }
}

trace_handler!(MemoryStore, GetIdea, Result<Idea, APIError>);

impl Handler<GetIdea> for MemoryStore {
    type Result = Result<Idea, APIError>;

    fn handle(&mut self, msg: GetIdea, _: &mut Self::Context) -> Self::Result {

        let is = self.ideas.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.collection)
            .ok_or_else(|| APIError::new(404, "Not Found", "The collection ID you provided could not be found. Please check it and try again."))
            .and_then(|c| 
                c.get(&msg.id).cloned()
                .ok_or_else(|| APIError::new(404, "Not Found", "The idea ID you provided could not be found. Please check it and try again.")))
            
    }
}

trace_handler!(MemoryStore, GetIdeas, Result<Vec<Idea>, APIError>);

impl Handler<GetIdeas> for MemoryStore {
    type Result = Result<Vec<Idea>, APIError>;

    fn handle(&mut self, msg: GetIdeas, _: &mut Self::Context) -> Self::Result {

        let is = self.ideas.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.collection)
            .ok_or_else(|| APIError::new(404, "Not Found", "The collection ID you provided could not be found. Please check it and try again."))
            .map(|items| items.iter().filter(|(_, i)| {
                if let Some(is_completed) = msg.is_completed {
                    if i.completed != is_completed {
                        return false;
                    }
                }

                if let Some(tag) = msg.tag.clone() {
                    if !i.tags.contains(tag.as_str()) {
                        return false;
                    }
                }

                true
            }).map(|(_id, idea)| idea.clone()).collect())
    }
}

trace_handler!(MemoryStore, GetRandomIdea, Result<Idea, APIError>);

impl Handler<GetRandomIdea> for MemoryStore {
    type Result = Result<Idea, APIError>;

    fn handle(&mut self, msg: GetRandomIdea, _: &mut Self::Context) -> Self::Result {

        let is = self.ideas.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.collection)
            .ok_or_else(|| APIError::new(404, "Not Found", "The collection ID you provided could not be found. Please check it and try again."))
            .and_then(|items| items.iter().filter(|(_, i)| {
                if let Some(is_completed) = msg.is_completed {
                    if i.completed != is_completed {
                        return false;
                    }
                }

                if let Some(tag) = msg.tag.clone() {
                    if !i.tags.contains(tag.as_str()) {
                        return false;
                    }
                }

                true
            }).choose(&mut rand::thread_rng())
                .ok_or_else(|| APIError::new(404, "Not Found", "No random ideas were available."))
                .map(|(_id, idea)| idea.clone()))
            
    }
}

trace_handler!(MemoryStore, StoreIdea, Result<Idea, APIError>);

impl Handler<StoreIdea> for MemoryStore {
    type Result = Result<Idea, APIError>;

    fn handle(&mut self, msg: StoreIdea, _: &mut Self::Context) -> Self::Result {

        let mut is = self.ideas.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        let idea = Idea {
            id: msg.id,
            collection_id: msg.collection,
            name: msg.name.clone(),
            description: msg.description.clone(),
            tags: msg.tags.clone(),
            completed: msg.completed,
        };
        
        is.entry(msg.collection)
            .or_insert_with(BTreeMap::new)
            .insert(idea.id, idea.clone());

        Ok(idea)
    }
}

trace_handler!(MemoryStore, RemoveIdea, Result<(), APIError>);

impl Handler<RemoveIdea> for MemoryStore {
    type Result = Result<(), APIError>;

    fn handle(&mut self, msg: RemoveIdea, _: &mut Self::Context) -> Self::Result {

        let mut is = self.ideas.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get_mut(&msg.collection)
            .ok_or_else(|| APIError::new(404, "Not Found", "The collection ID you provided could not be found. Please check it and try again."))
            .and_then(|c|
                c.remove(&msg.id)
                .map(|_| ())
                .ok_or_else(|| APIError::new(404, "Not Found", "The idea ID you provided could not be found. Please check it and try again.")))
            
    }
}

trace_handler!(MemoryStore, GetCollection, Result<Collection, APIError>);

impl Handler<GetCollection> for MemoryStore {
    type Result = Result<Collection, APIError>;

    fn handle(&mut self, msg: GetCollection, _: &mut Self::Context) -> Self::Result {

        let is = self.collections.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.principal_id)
            .ok_or_else(|| APIError::new(404, "Not Found", "The principal ID you provided could not be found. This likely means that you do not yet have any collections."))
            .and_then(|c| 
                c.get(&msg.id).cloned()
                .ok_or_else(|| APIError::new(404, "Not Found", "The collection ID you provided could not be found. Please check it and try again.")))
            
    }
}

trace_handler!(MemoryStore, GetCollections, Result<Vec<Collection>, APIError>);

impl Handler<GetCollections> for MemoryStore {
    type Result = Result<Vec<Collection>, APIError>;

    fn handle(&mut self, msg: GetCollections, _: &mut Self::Context) -> Self::Result {
        let is = self.collections.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.principal_id)
            .ok_or_else(|| APIError::new(404, "Not Found", "The principal ID you provided could not be found. This probably means that you do not yet have any collections."))
            .map(|items| items.iter()
                .map(|(_id, collection)| collection.clone()).collect())
    }
}

trace_handler!(MemoryStore, StoreCollection, Result<Collection, APIError>);

impl Handler<StoreCollection> for MemoryStore {
    type Result = Result<Collection, APIError>;

    fn handle(&mut self, msg: StoreCollection, _: &mut Self::Context) -> Self::Result {

        let mut is = self.collections.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        let collection = Collection {
            collection_id: msg.collection_id,
            user_id: msg.principal_id,
            name: msg.name.clone(),
        };
        
        is.entry(msg.principal_id)
            .or_insert_with(BTreeMap::new)
            .insert(collection.collection_id, collection.clone());

        Ok(collection)
    }
}

trace_handler!(MemoryStore, RemoveCollection, Result<(), APIError>);

impl Handler<RemoveCollection> for MemoryStore {
    type Result = Result<(), APIError>;

    fn handle(&mut self, msg: RemoveCollection, _: &mut Self::Context) -> Self::Result {

        let mut is = self.collections.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get_mut(&msg.principal_id)
            .ok_or_else(|| APIError::new(404, "Not Found", "The principal ID you provided could not be found. This likely means that you do not yet have any collections."))
            .and_then(|c|
                c.remove(&msg.id)
                .map(|_| ())
                .ok_or_else(|| {
                    debug!("Could not find a collection entry for {} in the current user's collection list ({}).", msg.id, msg.principal_id);
                    APIError::new(404, "Not Found", "The collection ID you provided could not be found. Please check it and try again.")
                }))
    }
}

trace_handler!(MemoryStore, GetRoleAssignment, Result<RoleAssignment, APIError>);

impl Handler<GetRoleAssignment> for MemoryStore {
    type Result = Result<RoleAssignment, APIError>;

    fn handle(&mut self, msg: GetRoleAssignment, _: &mut Self::Context) -> Self::Result {

        let is = self.role_assignments.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.collection_id)
            .ok_or_else(|| APIError::new(403, "Forbidden", "You do not have permission to access this resource."))
            .and_then(|c| 
                c.get(&msg.principal_id).cloned()
                .ok_or_else(|| APIError::new(403, "Forbidden", "You do not have permission to access this resource.")))
            
    }
}

trace_handler!(MemoryStore, GetRoleAssignments, Result<Vec<RoleAssignment>, APIError>);

impl Handler<GetRoleAssignments> for MemoryStore {
    type Result = Result<Vec<RoleAssignment>, APIError>;

    fn handle(&mut self, msg: GetRoleAssignments, _: &mut Self::Context) -> Self::Result {
        let is = self.role_assignments.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.collection_id)
            .ok_or_else(|| {
                debug!("Could not find a collection entry for {} in role assignments.", msg.collection_id);
                APIError::new(404, "Not Found", "The collection ID you provided could not be found. Please check it and try again.")
            })
            .map(|items| items.iter()
                .map(|(_id, collection)| collection.clone()).collect())
    }
}

trace_handler!(MemoryStore, StoreRoleAssignment, Result<RoleAssignment, APIError>);

impl Handler<StoreRoleAssignment> for MemoryStore {
    type Result = Result<RoleAssignment, APIError>;

    fn handle(&mut self, msg: StoreRoleAssignment, _: &mut Self::Context) -> Self::Result {

        let mut is = self.role_assignments.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        let role_assignment = RoleAssignment {
            collection_id: msg.collection_id,
            user_id: msg.principal_id,
            role: msg.role,
        };
        
        is.entry(msg.collection_id)
            .or_insert_with(BTreeMap::new)
            .insert(role_assignment.user_id, role_assignment.clone());

        Ok(role_assignment)
    }
}

trace_handler!(MemoryStore, RemoveRoleAssignment, Result<(), APIError>);

impl Handler<RemoveRoleAssignment> for MemoryStore {
    type Result = Result<(), APIError>;

    fn handle(&mut self, msg: RemoveRoleAssignment, _: &mut Self::Context) -> Self::Result {

        let mut is = self.role_assignments.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get_mut(&msg.collection_id)
        .ok_or_else(|| {
            debug!("Could not find a collection entry for {} in role assignments.", msg.collection_id);
            APIError::new(404, "Not Found", "The collection ID you provided could not be found. Please check it and try again.")
        })
            .and_then(|c|
                c.remove(&msg.principal_id)
                .map(|_| ())
                .ok_or_else(|| {
                    debug!("Could not find an entry for the user {} in the collection role assignments table for {}", msg.principal_id, msg.collection_id);
                    APIError::new(404, "Not Found", "The principal ID you provided could not be found. This likely means that you do not yet have any collections.")
                }))
    }
}

trace_handler!(MemoryStore, GetUser, Result<User, APIError>);

impl Handler<GetUser> for MemoryStore {
    type Result = Result<User, APIError>;

    fn handle(&mut self, msg: GetUser, _: &mut Self::Context) -> Self::Result {
        let users = self.users.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        users.get(&msg.email_hash)
            .ok_or_else(|| APIError::new(404, "Not Found", "No user could be found with the email hash you provided. Please check it and try again."))
            .map(|u| u.clone())
    }
}

trace_handler!(MemoryStore, StoreUser, Result<User, APIError>);

impl Handler<StoreUser> for MemoryStore {
    type Result = Result<User, APIError>;

    fn handle(&mut self, msg: StoreUser, _: &mut Self::Context) -> Self::Result {
        let mut users = self.users.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        let user = User {
            principal_id: msg.principal_id,
            email_hash: msg.email_hash,
            first_name: msg.first_name.clone()
        };

        users.insert(msg.email_hash, user.clone());

        Ok(user)
    }
}