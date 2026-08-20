//! The handle every handler receives.
//!
//! Handlers are generic over [`Services`] rather than taking a concrete struct,
//! which is what lets the test suite swap [`MemoryStore`] in for
//! [`SqliteStore`](crate::db::SqliteStore) structurally — no branches in
//! shipping code, no `#[cfg(test)]` in the request path.

use std::sync::Arc;

use crate::{auth::OidcState, config::Config, db::Store};

pub trait Services: Clone + Send + Sync + 'static {
    type Store: Store;

    fn config(&self) -> &Arc<Config>;
    fn store(&self) -> &Self::Store;
    fn oidc(&self) -> &Arc<OidcState>;
}

#[derive(Clone)]
pub struct ServicesContainer<S: Store> {
    config: Arc<Config>,
    store: S,
    oidc: Arc<OidcState>,
}

impl<S: Store> ServicesContainer<S> {
    pub fn new(config: Arc<Config>, store: S, oidc: Arc<OidcState>) -> Self {
        Self {
            config,
            store,
            oidc,
        }
    }
}

impl<S: Store> Services for ServicesContainer<S> {
    type Store = S;

    fn config(&self) -> &Arc<Config> {
        &self.config
    }

    fn store(&self) -> &Self::Store {
        &self.store
    }

    fn oidc(&self) -> &Arc<OidcState> {
        &self.oidc
    }
}
