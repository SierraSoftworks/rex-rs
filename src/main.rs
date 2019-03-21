#![feature(proc_macro_hygiene, decl_macro, impl_trait_in_bindings, const_fn)]

extern crate actix_web;
extern crate chrono;
#[macro_use]
extern crate sentry;
extern crate sentry_actix;
#[macro_use]
extern crate serde;
extern crate rand;
extern crate serde_json;
extern crate uuid;

mod api;
mod errors;
mod health;
mod ideas;
mod stator;

use actix_web::middleware::cors::Cors;
use actix_web::{server, App, Error};
use sentry_actix::SentryMiddleware;
use std::sync::Arc;

fn main() -> Result<(), Error> {
    let raven = sentry::init((
        "https://b7ca8a41e8e84fef889e4f428071dab2@sentry.io/1415519",
        sentry::ClientOptions {
            release: release_name!(),
            ..Default::default()
        },
    ));

    if raven.is_enabled() {
        sentry::integrations::panic::register_panic_handler();
    }

    let mut state: state::Container = state::Container::new();

    state.set(health::new_state());
    state.set(ideas::new_state());

    state.freeze();

    let state_arc = Arc::new(state);

    server::new(move || {
        App::with_state(state_arc.clone())
            // .configure(|app| Cors::for_app(app).register())
            .configure(ideas::configure)
            .configure(health::configure)
            .middleware(SentryMiddleware::new())
    })
    .bind("127.0.0.1:8000")?
    .run();

    Ok(())
}
