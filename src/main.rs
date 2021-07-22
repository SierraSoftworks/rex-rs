extern crate actix_web;
extern crate chrono;
#[macro_use] extern crate serde;
extern crate rand;
extern crate serde_json;
extern crate uuid;
#[macro_use] extern crate tracing;
#[macro_use] extern crate sentry;
#[macro_use] extern crate lazy_static;

#[macro_use] mod macros;

mod api;
mod models;
mod store;
mod telemetry;

use actix_web::{App, HttpServer};
use telemetry::{Session, TracingLogger};

fn get_listening_port() -> u16 {
    std::env::var("FUNCTIONS_CUSTOMHANDLER_PORT").map(|v| v.parse().unwrap_or(8000)).unwrap_or(8000)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let _session = Session::new();

    let _raven = sentry::init((
        "https://b7ca8a41e8e84fef889e4f428071dab2@sentry.io/1415519",
        sentry::ClientOptions {
            release: release_name!(),
            ..Default::default()
        }
    ));

    let state = models::GlobalState::new();

    info!("Starting server on :{}", get_listening_port());
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(state.clone()))
            .wrap(TracingLogger)
            // TODO: CORS needs to be updated for new actix-web
            // .wrap(Cors::default().allow_any_origin().send_wildcard())
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
