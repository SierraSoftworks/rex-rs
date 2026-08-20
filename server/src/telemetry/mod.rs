//! Observability: Sentry, OpenTelemetry, and analytics, all driven by config.

mod tracing_logger;

pub use tracing_logger::TracingLogger;

use tracing_batteries::{Medama, OpenTelemetry, Sentry, Session};

use crate::config::TelemetryConfig;

#[cfg(not(debug_assertions))]
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(debug_assertions)]
const VERSION: &str = "0.0.0-dev";

/// A running telemetry session, or the local fallback when nothing is
/// configured to report to.
pub enum Telemetry {
    Reporting(Session),
    /// Logs to stderr and goes no further — the shape of a development run.
    Local,
}

impl Telemetry {
    pub fn record_error<E: std::error::Error>(&self, error: &E) {
        if let Telemetry::Reporting(session) = self {
            session.record_error(error);
        }
    }

    pub fn shutdown(self) {
        if let Telemetry::Reporting(session) = self {
            session.shutdown();
        }
    }
}

/// Starts whichever batteries the configuration asks for.
///
/// Each battery stays switched off while its setting is absent, so a
/// development instance reports to nothing at all without needing a special
/// code path — it just logs to stderr instead.
pub fn setup(config: &TelemetryConfig) -> Telemetry {
    let metadata = Session::new("rex", VERSION);
    let mut session: Option<Session> = None;

    // The first battery is what turns the metadata into a session, so each
    // attachment has to cope with either half of that transition.
    macro_rules! attach {
        ($battery:expr) => {
            session = Some(match session.take() {
                Some(session) => session.with_battery($battery),
                None => metadata.clone().with_battery($battery),
            });
        };
    }

    if let Some(dsn) = config.sentry_dsn.as_deref() {
        attach!(Sentry::new(dsn));
    }

    if let Some(endpoint) = config.otlp_endpoint.as_deref() {
        let mut otlp = OpenTelemetry::new(endpoint.to_string());

        for (name, value) in &config.otlp_headers {
            otlp = otlp.with_header(name.clone(), value.clone());
        }

        attach!(otlp);
    }

    if let Some(endpoint) = config.analytics_endpoint.as_deref() {
        attach!(Medama::new(endpoint.to_string()));
    }

    match session {
        Some(session) => Telemetry::Reporting(session),
        None => {
            // Nothing to export to, but the process still needs a subscriber or
            // every `info!` in the codebase goes nowhere.
            let _ = tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                )
                .try_init();

            Telemetry::Local
        }
    }
}
