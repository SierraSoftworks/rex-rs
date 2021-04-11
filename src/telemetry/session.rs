use opentelemetry::{KeyValue, sdk};
use tracing_subscriber::{Registry, prelude::__tracing_subscriber_SubscriberExt};

pub struct Session {
}

impl Session {
    pub fn new() -> Self {
        tracing_log::LogTracer::init().expect("log tracer should be installed");

        let app_insights_key = std::env::var("APPINSIGHTS_INSTRUMENTATIONKEY").unwrap_or_default();
        if app_insights_key.is_empty()
        {
            let tracer = sdk::export::trace::stdout::new_pipeline()
                .install_simple();

            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
            let subscriber = Registry::default().with(telemetry);

            tracing::subscriber::set_global_default(subscriber).unwrap_or_default();

            Self {
            }
        } else {
            let tracer = opentelemetry_application_insights::new_pipeline(
                app_insights_key
            )
                .with_client(reqwest::Client::new())
                .with_trace_config(sdk::trace::config().with_resource(sdk::Resource::new(vec![
                    KeyValue::new("service.name", "Rex"),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION"))
                ])))
                .install_batch(opentelemetry::runtime::Tokio);

            let telemetry = tracing_opentelemetry::layer()
                .with_tracked_inactivity(true)
                .with_tracer(tracer);

            let subscriber = Registry::default().with(telemetry);
            tracing::subscriber::set_global_default(subscriber).unwrap_or_default();

            Self {
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}