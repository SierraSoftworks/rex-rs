use filt_rs::{Filter, FilterValue, Filterable};
use tracing::error;

use super::AuthToken;

/// A compiled deny-by-default access gate.
///
/// An absent ACL means "no opinion" — the caller decides what that implies —
/// while a present one must evaluate truthy for access to be granted. An ACL
/// that fails to evaluate denies, because the alternative is a filter typo
/// silently opening the door.
#[derive(Clone)]
pub struct Acl {
    source: String,
    filter: Filter,
}

impl Acl {
    pub fn new(source: &str) -> Result<Self, filt_rs::Error> {
        Ok(Self {
            source: source.to_string(),
            filter: Filter::new(source)?,
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn allows(&self, context: &AclContext<'_>) -> bool {
        match self.filter.matches(context) {
            Ok(allowed) => allowed,
            Err(err) => {
                error!(
                    { exception.message = %err, acl = %self.source },
                    "An access control expression could not be evaluated; denying access."
                );
                false
            }
        }
    }
}

/// What an ACL expression may refer to.
pub struct AclContext<'a> {
    pub token: &'a AuthToken,
    pub client_ip: Option<&'a str>,
    pub method: &'a str,
    pub path: &'a str,
}

impl Filterable for AclContext<'_> {
    fn get(&self, key: &str) -> FilterValue<'_> {
        match key {
            "client_ip" => self.client_ip.into(),
            "method" => self.method.into(),
            "path" => self.path.into(),
            other => match other.strip_prefix("claims.") {
                Some(claim) => self.token.claim(claim).into(),
                None => FilterValue::Null,
            },
        }
    }
}
