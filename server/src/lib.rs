//! Rex: a self-contained binary serving the Rex API and its web UI.
//!
//! One process listens on one port, keeps its data in SQLite, authenticates
//! against any standard OIDC provider, and serves a Yew single-page application
//! embedded in the executable itself.

pub mod auth;
pub mod config;
pub mod db;
pub mod models;
pub mod services;
pub mod telemetry;
pub mod web;

#[cfg(test)]
pub mod testing;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web as actix};
use tracing::{info, warn};

use crate::{
    auth::OidcState,
    config::Config,
    db::{MemoryStore, SqliteStore, Store},
    services::ServicesContainer,
    telemetry::TracingLogger,
};

/// Opens the configured store and serves until the process is asked to stop.
pub async fn run(config: Config) -> std::io::Result<()> {
    let config = Arc::new(config);

    let oidc = Arc::new(
        OidcState::new(&config)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?,
    );

    // The store is chosen at startup but baked into the type of everything
    // downstream, so each branch runs its own monomorphised server.
    if config.web.database == "memory" {
        warn!(
            "Rex is running with an in-memory store; everything it holds is lost when the process exits."
        );
        serve(config.clone(), MemoryStore::new(), oidc).await
    } else {
        let store = SqliteStore::open(&config.web.database)
            .await
            .map_err(|err| std::io::Error::other(err.to_string()))?;

        serve(config.clone(), store, oidc).await
    }
}

async fn serve<S: Store>(
    config: Arc<Config>,
    store: S,
    oidc: Arc<OidcState>,
) -> std::io::Result<()> {
    let address = config.web.address.clone();
    let services = ServicesContainer::new(config.clone(), store, oidc.clone());

    info!("Starting the Rex server on {}.", address);

    HttpServer::new(move || {
        // Same-origin UI and API mean CORS is normally unnecessary; it exists
        // for the cutover window, while the old statically hosted UI may still
        // be pointing here.
        let mut cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        for origin in &config.web.cors_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .app_data(actix::Data::new(services.clone()))
            .app_data(actix::Data::new(oidc.clone()))
            .wrap(cors)
            .wrap(TracingLogger)
            .configure(web::configure::<ServicesContainer<S>>)
    })
    .bind(address)?
    .run()
    .await
}
