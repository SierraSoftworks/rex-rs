extern crate actix_web;
extern crate chrono;
#[macro_use]
extern crate serde;
extern crate rand;
extern crate serde_json;
extern crate uuid;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;

mod api;
mod models;
mod store;
mod telemetry;
mod ui;

use actix_web::{App, HttpServer};
use telemetry::TracingLogger;

fn get_listening_port() -> u16 {
    std::env::var("FUNCTIONS_CUSTOMHANDLER_PORT")
        .map(|v| v.parse().unwrap_or(8000))
        .unwrap_or(8000)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let session = telemetry::setup();

    let state = models::GlobalState::new();

    info!("Starting server on :{}", get_listening_port());
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(state.clone()))
            .wrap(TracingLogger)
            // TODO: CORS needs to be updated for new actix-web
            // .wrap(Cors::default().allow_any_origin().send_wildcard())
            .configure(api::configure)
            .configure(ui::configure)
    })
    .bind(format!("0.0.0.0:{}", get_listening_port()))?
    .run()
    .await
    .map_err(|err| {
        error!("The server exited unexpectedly: {}", err);
        session.record_error(&err);

        err
    })
}
