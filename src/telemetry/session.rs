use opentelemetry_otlp::WithExportConfig;
use sentry::ClientInitGuard;
use tracing_subscriber::prelude::*;

pub struct Session {
    raven: ClientInitGuard,
}

impl Session {
    pub fn new() -> Self {
        let raven = sentry::init((
            "https://b7ca8a41e8e84fef889e4f428071dab2@o219072.ingest.sentry.io/1415519",
            sentry::ClientOptions {
                release: Some(version!("rex@v").into()),
                #[cfg(debug_assertions)]
                environment: Some("Development".into()),
                #[cfg(not(debug_assertions))]
                environment: Some("Production".into()),
                default_integrations: true,
                attach_stacktrace: true,
                send_default_pii: false,
                ..Default::default()
            },
        ));

        #[cfg(not(debug_assertions))]
        let honeycomb_key = std::env::var("HONEYCOMB_KEY").ok();

        #[cfg(debug_assertions)]
        let honeycomb_key = Some("X6naTEMkzy10PMiuzJKifF");

        if let Some(honeycomb_key) = honeycomb_key {
            let mut tracing_metadata = tonic::metadata::MetadataMap::new();
            tracing_metadata.insert("x-honeycomb-team", honeycomb_key.parse().unwrap());

            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint("https://api.honeycomb.io:443")
                        .with_metadata(tracing_metadata),
                )
                .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
                    opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", "rex"),
                        opentelemetry::KeyValue::new("service.version", version!("v")),
                        opentelemetry::KeyValue::new("host.os", std::env::consts::OS),
                        opentelemetry::KeyValue::new("host.architecture", std::env::consts::ARCH),
                    ]),
                ))
                .install_batch(opentelemetry_sdk::runtime::Tokio)
                .unwrap();

            tracing_subscriber::registry()
                .with(tracing_subscriber::filter::LevelFilter::DEBUG)
                .with(tracing_subscriber::filter::dynamic_filter_fn(
                    |_metadata, ctx| {
                        !ctx.lookup_current()
                            // Exclude the rustls session "Connection" events which don't have a parent span
                            .map(|s| s.parent().is_none() && s.name() == "Connection")
                            .unwrap_or_default()
                    },
                ))
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .init();
        }

        sentry::start_session();

        Self { raven }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
        self.raven.close(None);
        sentry::end_session();
    }
}
