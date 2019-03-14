#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;
extern crate chrono;
extern crate rand;
extern crate sentry;
extern crate uuid;

mod api;
mod errors;
mod health;
mod ideas;

fn app() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/api",
            routes![
                health::health_v1,
                health::health_v2,
                ideas::ideas_v1,
                ideas::idea_v1,
                ideas::new_idea_v1,
                ideas::random_idea_v1,
                ideas::ideas_v2,
                ideas::idea_v2,
                ideas::new_idea_v2,
                ideas::random_idea_v2,
            ],
        )
        .register(catchers![
            errors::error_404,
            errors::error_422,
            errors::error_500,
        ])
        .manage(health::new_state())
        .manage(ideas::new_state())
}

fn main() {
    let raven = sentry::init("https://b7ca8a41e8e84fef889e4f428071dab2@sentry.io/1415519");

    if raven.is_enabled() {
        sentry::integrations::panic::register_panic_handler();
    }

    app().launch();
}
