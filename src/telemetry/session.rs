use opentelemetry::{KeyValue, sdk};
use tracing_subscriber::{Registry, prelude::__tracing_subscriber_SubscriberExt};

pub struct Session {
    _uninstall_stdout: Option<sdk::export::trace::stdout::Uninstall>,
    _uninstall_appinsights: Option<opentelemetry_application_insights::Uninstall>,
}

impl Session {
    pub fn new() -> Self {
        let app_insights_key = std::env::var("APPINSIGHTS_INSTRUMENTATIONKEY").unwrap_or_default();
        if app_insights_key.is_empty()
        {
            let (tracer, uninstall) = sdk::export::trace::stdout::new_pipeline()
                .install();

            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
            let subscriber = Registry::default().with(telemetry);

            tracing::subscriber::set_global_default(subscriber).unwrap_or_default();

            // tracing_subscriber::fmt()
            //     .with_max_level(tracing::Level::INFO)
            //     .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            //     .init();

            Self {
                _uninstall_stdout: Some(uninstall),
                _uninstall_appinsights: None
            }
        } else {
            let (tracer, uninstall) = opentelemetry_application_insights::new_pipeline(
                app_insights_key
            )
                .with_client(reqwest::Client::new())
                .with_trace_config(sdk::trace::config().with_resource(sdk::Resource::new(vec![
                    KeyValue::new("service.name", "Rex"),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION"))
                ])))
                .install();

            let telemetry = tracing_opentelemetry::layer()
                .with_tracked_inactivity(true)
                .with_tracer(tracer);

            let subscriber = Registry::default().with(telemetry);
            tracing::subscriber::set_global_default(subscriber).unwrap_or_default();

            Self {
                _uninstall_stdout: None,
                _uninstall_appinsights: Some(uninstall),
            }
        }
    }
}