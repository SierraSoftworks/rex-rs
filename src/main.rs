extern crate actix_web;
extern crate chrono;
#[macro_use] extern crate serde;
extern crate rand;
extern crate serde_json;
extern crate uuid;
#[macro_use] extern crate log;
#[macro_use] extern crate sentry;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate prometheus;

#[macro_use] mod macros;

mod api;
mod models;
mod store;

use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};
use actix_web_prom::PrometheusMetrics;

fn get_listening_port() -> u16 {
    std::env::var("FUNCTIONS_CUSTOMHANDLER_PORT").map(|v| v.parse().unwrap_or(8000)).unwrap_or(8000)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let _raven = sentry::init((
        "https://b7ca8a41e8e84fef889e4f428071dab2@sentry.io/1415519",
        sentry::ClientOptions {
            release: release_name!(),
            ..Default::default()
        }
        .add_integration(sentry::integrations::log::LogIntegration::default()),
    ));

    let state = models::GlobalState::new();
    let metrics = PrometheusMetrics::new_with_registry(prometheus::default_registry().clone(), "rex", Some("/api/v1/metrics"), None).unwrap();

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(metrics.clone())
            .wrap(middleware::Logger::default())
            .wrap(Cors::new().send_wildcard().finish())
            .configure(api::configure)
    })
        .bind(format!("0.0.0.0:{}", get_listening_port()))?
        .run()
        .await
        .map_err(|err| {
            error!("The server exited unexpectedly: {}", err);
            sentry::capture_event(sentry::protocol::Event {
                message: Some(format!("Server Exited Unexpectedly: {}", err).into()),
                level: sentry::protocol::Level::Fatal,
                ..Default::default()
            });

            err
        })
}
