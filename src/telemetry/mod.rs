mod actix_message;
mod actix_web_tracing;

pub use actix_message::*;
pub use actix_web_tracing::TracingLogger;

use tracing_batteries::{Medama, OpenTelemetry, Sentry, Session};

pub fn setup() -> Session {
    Session::new("rex", version!())
        .with_battery(Sentry::new(
            "https://b7ca8a41e8e84fef889e4f428071dab2@o219072.ingest.sentry.io/1415519",
        ))
        .with_battery(
            OpenTelemetry::new("https://api.honeycomb.io:443").with_header(
                "x-honeycomb-team",
                std::env::var("HONEYCOMB_KEY").unwrap_or_default(),
            ),
        )
        .with_battery(Medama::new("https://analytics.sierrasoftworks.com"))
}
