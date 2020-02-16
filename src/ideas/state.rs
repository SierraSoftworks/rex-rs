use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use super::super::api;
use super::models;

#[derive(Clone)]
pub struct IdeasState {
    pub store: Arc<RwLock<BTreeMap<u128, models::Idea>>>,
}

impl IdeasState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn ideas<T: api::StateView<models::Idea>>(&self) -> Option<Vec<T>> {
        self.store.read().ok().and_then(|store| {
            Some(
                store
                    .iter()
                    .map(|(_id, idea)| T::from_state(idea))
                    .collect::<Vec<_>>(),
            )
        })
    }
    pub fn idea<T: api::StateView<models::Idea>>(&self, id: u128) -> Option<T> {
        self.store
            .read()
            .ok()
            .and_then(|store| store.get(&id).and_then(|idea| Some(T::from_state(idea))))
    }
    pub fn store_idea<T: api::StateView<models::Idea>>(&self, idea: &T) -> Option<T> {
        self.store.write().ok().and_then(|mut store| {
            let idea = idea.to_state();
            let id = idea.id;
            store.insert(id, idea);
            Some(T::from_state(
                store.get(&id).expect("idea to be in the store"),
            ))
        })
    }

    pub fn ideas_by<T: api::StateView<models::Idea>>(
        &self,
        pred: impl Fn(&models::Idea) -> bool,
    ) -> Option<Vec<T>> {
        self.store.read().ok().and_then(|store| {
            Some(
                store
                    .iter()
                    .filter(|(_id, idea)| pred(idea))
                    .map(|(_id, idea)| T::from_state(idea))
                    .collect::<Vec<_>>(),
            )
        })
    }

    pub fn random_idea<T: api::StateView<models::Idea>>(
        &self,
        pred: impl Fn(&models::Idea) -> bool,
    ) -> Option<T> {
        self.store.read().ok().and_then(|store| {
            let ids = store
                .iter()
                .filter(|(_id, idea)| pred(idea))
                .map(|(id, _idea)| id)
                .collect::<Vec<_>>();
            if ids.len() > 0 {
                let mut rng = rand::thread_rng();
                let index = rand::seq::index::sample(&mut rng, ids.len(), 1).index(0);
                let id = ids[index];
                store.get(&id).map(|item| T::from_state(item))
            } else {
                None
            }
        })
    }
}
