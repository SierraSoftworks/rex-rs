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

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
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

    let ideas_state = web::Data::new(ideas::IdeasState::new());
    let health_state = web::Data::new(health::HealthState::new());

    HttpServer::new(move || {
        App::new()
            .app_data(ideas_state.clone())
            .app_data(health_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(Cors::new().send_wildcard().allowed_origin("All").finish())
            .configure(ideas::configure)
            .configure(health::configure)
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
