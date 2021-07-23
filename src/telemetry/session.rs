use opentelemetry::{sdk, KeyValue};
use tracing::{metadata::LevelFilter};
use tracing_honeycomb::new_honeycomb_telemetry_layer;
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::*, Registry};

pub struct Session {}

impl Session {
    pub fn new() -> Self {
        LogTracer::init().unwrap();

        let honeycomb_layer = match std::env::var("HONEYCOMB_KEY").ok() {
            Some(honeycomb_key) if !honeycomb_key.is_empty() => {
                let config = libhoney::Config {
                    options: libhoney::client::Options {
                        api_key: honeycomb_key,
                        dataset:  std::env::var("HONEYCOMB_DATASET").unwrap_or("rex.sierrasoftworks.com".into()),
                        ..libhoney::client::Options::default()
                    },
                    transmission_options: libhoney::transmission::Options::default(),
                };

                let layer = new_honeycomb_telemetry_layer("rex", config);
                Some(layer)
            }
            _ => None,
        };

        let app_insights_layer = match std::env::var("APPINSIGHTS_INSTRUMENTATIONKEY").ok() {
            Some(app_insights_key) if !app_insights_key.is_empty() => {
                let tracer = opentelemetry_application_insights::new_pipeline(app_insights_key)
                    .with_client(reqwest::Client::new())
                    .with_trace_config(sdk::trace::config().with_resource(sdk::Resource::new(
                        vec![
                            KeyValue::new("service.name", "Rex"),
                            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                        ],
                    )));

                #[cfg(test)]
                let tracer = tracer.install_simple();

                #[cfg(not(test))]
                let tracer = tracer.install_batch(opentelemetry::runtime::Tokio);

                let layer = tracing_opentelemetry::layer()
                    .with_tracked_inactivity(true)
                    .with_tracer(tracer);

                Some(layer)
            }
            _ => None,
        };

        let default_layer = tracing_subscriber::fmt::Layer::default();

        let subscriber = Registry::default().with(LevelFilter::INFO);

        match (honeycomb_layer, app_insights_layer, default_layer) {
            (Some(hny), Some(_ai), default) => {
                let subscriber = subscriber
                    .with(default)
                    .with(hny);

                tracing::subscriber::set_global_default(subscriber).unwrap_or_default();
            }
            (Some(hny), None, default) => {
                let subscriber = subscriber
                    .with(default)
                    .with(hny);

                tracing::subscriber::set_global_default(subscriber).unwrap_or_default();
            }
            (None, Some(ai), default) => {
                let subscriber = subscriber
                    .with(default)
                    .with(ai);

                tracing::subscriber::set_global_default(subscriber).unwrap_or_default();
            }
            (None, None, default) => {
                let subscriber = subscriber.with(default);

                tracing::subscriber::set_global_default(subscriber).unwrap_or_default();
            }
        }

        Self {}
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}
