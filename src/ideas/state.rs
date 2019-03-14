use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use super::super::api;
use super::models;

pub fn new_state() -> IdeasState {
    IdeasState {
        store: Arc::new(RwLock::new(BTreeMap::new())),
    }
}

pub struct IdeasState {
    pub store: Arc<RwLock<BTreeMap<u128, models::Idea>>>,
}

pub fn ideas<T: api::StateView<models::Idea>>(state: &IdeasState) -> Option<Vec<T>> {
    state.store.read().ok().and_then(|store| {
        Some(
            store
                .iter()
                .map(|(_id, idea)| T::from_state(idea))
                .collect::<Vec<_>>(),
        )
    })
}

pub fn idea<T: api::StateView<models::Idea>>(id: u128, state: &IdeasState) -> Option<T> {
    state
        .store
        .read()
        .ok()
        .and_then(|store| store.get(&id).and_then(|idea| Some(T::from_state(idea))))
}

pub fn new_idea<T: api::StateView<models::Idea>>(new_idea: &T, state: &IdeasState) -> Option<u128> {
    state.store.write().ok().and_then(|mut store| {
        let idea = new_idea.to_state();
        let id = idea.id;

        store.insert(id, idea);

        Some(id)
    })
}

pub fn ideas_by<T: api::StateView<models::Idea>>(
    pred: impl Fn(&models::Idea) -> bool,
    state: &IdeasState,
) -> Option<Vec<T>> {
    state.store.read().ok().and_then(|store| {
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
    pred: impl Fn(&models::Idea) -> bool,
    state: &IdeasState,
) -> Option<T> {
    state.store.read().ok().and_then(|store| {
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
