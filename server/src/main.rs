use std::path::PathBuf;

use rex_server::{config::Config, run, telemetry};
use tracing::{error, info};

/// Where the configuration lives, unless `REX_CONFIG` says otherwise.
const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // A `.env` file is a convenience for development; the interpolation in
    // config.toml reads whatever ends up in the environment either way.
    let _ = dotenvy::dotenv();

    let path = std::env::var("REX_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_CONFIG_PATH));

    let config = match Config::load(&path) {
        Ok(config) => config,
        Err(rex_server::config::ConfigError::Read(err))
            if err.kind() == std::io::ErrorKind::NotFound =>
        {
            // Running with no configuration at all is a legitimate first
            // experience: it gets you a working server on the defaults.
            Config::default()
        }
        Err(err) => {
            eprintln!("{err}");
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, err));
        }
    };

    let session = telemetry::setup(&config.telemetry);

    if !config.auth_enabled() {
        info!("No identity provider is configured, so Rex is running without authentication.");
    }

    let result = run(config).await;

    if let Err(err) = &result {
        error!("The server exited unexpectedly: {}", err);
        session.record_error(err);
    }

    result
}
