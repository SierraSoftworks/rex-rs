use opentelemetry_application_insights::Uninstall;
use tracing::dispatcher::DefaultGuard;
use tracing_subscriber::{Registry, prelude::__tracing_subscriber_SubscriberExt};

pub struct Session {
    _guard: Option<DefaultGuard>,
    _uninstall: Option<Uninstall>,
}

impl Session {
    pub fn new() -> Self {
        let app_insights_key = std::env::var("APPINSIGHTS_INSTRUMENTATIONKEY").unwrap_or_default();
        if app_insights_key.is_empty()
        {
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
                .init();

            Self {
                _guard: None,
                _uninstall: None
            }
        } else {
            let (tracer, uninstall) = opentelemetry_application_insights::new_pipeline(
                app_insights_key
            )
                .with_client(reqwest::Client::new())
                .with_endpoint("https://northeurope-0.in.applicationinsights.azure.com/").expect("The AppInsights telemetry endpoint should parse correctly")
                .install();
        
            let telemetry = tracing_opentelemetry::layer()
                .with_tracer(tracer);
        
            let subscriber = Registry::default().with(telemetry);
            let guard = tracing::subscriber::set_default(subscriber);
    
            Self {
                _guard: Some(guard),
                _uninstall: Some(uninstall),
            }
        }
    }
}