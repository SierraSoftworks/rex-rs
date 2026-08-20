//! Configuration, loaded from one `config.toml`.
//!
//! Everything that used to be a hard-coded constant or a stray environment
//! variable lives here. Two rules are borrowed wholesale because they are cheap
//! and permanent: unknown keys are a startup error (`deny_unknown_fields`), and
//! `config.example.toml` is parsed by a unit test so the documentation cannot
//! drift away from the schema.

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ConfigError {
    Read(std::io::Error),
    Parse(toml::de::Error),
    Interpolation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Read(err) => write!(f, "We could not read the configuration file: {err}"),
            ConfigError::Parse(err) => {
                write!(f, "We could not parse the configuration file: {err}")
            }
            ConfigError::Interpolation(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Read(err) => Some(err),
            ConfigError::Parse(err) => Some(err),
            ConfigError::Interpolation(_) => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub web: WebConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct WebConfig {
    /// The socket the server listens on.
    pub address: String,

    /// Path to the SQLite database, or the literal `"memory"` for a throwaway
    /// in-process store (handy for `cargo run` without any setup).
    pub database: String,

    /// The externally visible origin, used to build absolute redirect URIs.
    pub base_url: Option<String>,

    /// Extra origins allowed to call the API cross-origin.
    ///
    /// Same-origin UI and API mean this is empty in a normal deployment; it
    /// exists for the cutover window, while the old statically hosted UI may
    /// still be pointing at the new server.
    pub cors_origins: Vec<String>,

    pub auth: AuthConfig,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:8000".into(),
            database: "rex.sqlite".into(),
            base_url: None,
            cors_origins: Vec::new(),
            auth: AuthConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AuthConfig {
    /// Absent means authentication is disabled entirely — useful for local
    /// development and the end-to-end suite, never for a real deployment.
    pub oidc: Option<OidcConfig>,

    /// Deny-by-default gate deciding who may sign in at all, evaluated over the
    /// validated token's claims (`claims.*`), `client_ip`, `method`, and
    /// `path`. Absent means "any principal with a valid token".
    pub user_acl: Option<String>,

    /// The same, for any administrative surface. Absent means nobody.
    pub admin_acl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OidcConfig {
    /// The issuer URL; discovery is `{endpoint}/.well-known/openid-configuration`.
    pub endpoint: String,
    pub client_id: String,
    pub client_secret: Option<String>,

    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,

    /// Which claim carries the principal id.
    ///
    /// Defaults to the standard `sub`. The Sierra Softworks deployment sets
    /// `oid`, because that is the claim its existing principal ids came from
    /// and changing it would detach every user from their data.
    #[serde(default = "default_username_claim")]
    pub username_claim: String,

    /// Which claim carries the email address.
    #[serde(default = "default_email_claim")]
    pub email_claim: String,

    /// How long a cached discovery document (and its JWKS) is trusted before
    /// being refetched.
    #[serde(default = "default_discovery_ttl")]
    pub discovery_ttl_seconds: u64,
}

fn default_scopes() -> Vec<String> {
    ["openid", "profile", "email", "offline_access"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn default_username_claim() -> String {
    "sub".into()
}

fn default_email_claim() -> String {
    "email".into()
}

fn default_discovery_ttl() -> u64 {
    3600
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct TelemetryConfig {
    pub sentry_dsn: Option<String>,
    pub otlp_endpoint: Option<String>,
    pub otlp_headers: HashMap<String, String>,
    pub analytics_endpoint: Option<String>,
}

impl Config {
    /// Reads a config file, interpolating `${{ env.NAME }}` references first.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let raw = std::fs::read_to_string(path).map_err(ConfigError::Read)?;
        Self::parse(&raw)
    }

    pub fn parse(raw: &str) -> Result<Self, ConfigError> {
        let interpolated = interpolate(raw, |name| std::env::var(name).ok())?;
        toml::from_str(&interpolated).map_err(ConfigError::Parse)
    }

    /// Whether the server should require (and validate) bearer tokens.
    pub fn auth_enabled(&self) -> bool {
        self.web.auth.oidc.is_some()
    }
}

/// Replaces every `${{ env.NAME }}` in `raw` with the value `lookup` returns.
///
/// An unset variable is an error rather than an empty string: a silently blank
/// client secret fails much later and much more confusingly than a refusal to
/// start.
fn interpolate(raw: &str, lookup: impl Fn(&str) -> Option<String>) -> Result<String, ConfigError> {
    const OPEN: &str = "${{";
    const CLOSE: &str = "}}";

    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;

    while let Some(start) = rest.find(OPEN) {
        out.push_str(&rest[..start]);
        let after = &rest[start + OPEN.len()..];

        let end = after.find(CLOSE).ok_or_else(|| {
            ConfigError::Interpolation(
                "An interpolation was opened with `${{` but never closed with `}}`.".into(),
            )
        })?;

        let expression = after[..end].trim();
        let name = expression.strip_prefix("env.").ok_or_else(|| {
            ConfigError::Interpolation(format!(
                "`{expression}` is not a supported interpolation; only `env.NAME` is understood."
            ))
        })?;

        let value = lookup(name).ok_or_else(|| {
            ConfigError::Interpolation(format!(
                "The environment variable `{name}` referenced by the configuration is not set."
            ))
        })?;

        out.push_str(&value);
        rest = &after[end + CLOSE.len()..];
    }

    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_environment_references() {
        let out = interpolate("a = \"${{ env.NAME }}\"", |name| {
            assert_eq!(name, "NAME");
            Some("value".into())
        })
        .expect("the interpolation should succeed");

        assert_eq!(out, "a = \"value\"");
    }

    #[test]
    fn unset_environment_reference_is_an_error() {
        interpolate("a = \"${{ env.MISSING }}\"", |_| None)
            .expect_err("an unset variable should refuse to start the server");
    }

    #[test]
    fn unknown_keys_are_rejected() {
        Config::parse("[web]\nnot_a_real_key = 1\n")
            .expect_err("an unknown key should be a startup error");
    }

    /// The example file doubles as the schema's documentation, so it has to
    /// keep parsing as the schema changes.
    #[test]
    fn example_config_parses() {
        let raw = include_str!("../../config.example.toml");

        // The variables the example refers to are not set in CI, so stand in
        // for them rather than requiring the environment to be primed.
        let substituted =
            interpolate(raw, |_| Some("example".into())).expect("placeholders should substitute");

        let config = Config::parse(&substituted).expect("the example config should parse");
        assert!(
            config.auth_enabled(),
            "the example config should document a complete OIDC setup"
        );
    }
}
