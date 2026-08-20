//! The internal domain model.
//!
//! These types are what the store speaks; the wire types in [`rex_api`] are
//! converted to and from them at the edge of each handler. Ids are `u128`
//! in memory and 32-character zero-padded lowercase hex on the wire, which is
//! exactly the format the v1/v2/v3 APIs have always used.

mod collection;
mod health;
mod idea;
mod role_assignment;
mod user;

pub use collection::*;
pub use health::*;
pub use idea::*;
pub use role_assignment::*;
pub use user::*;

use rex_api::ApiError;

/// Every entity in Rex is keyed by one of these.
pub type Id = u128;

pub fn new_id() -> Id {
    u128::from_be_bytes(*uuid::Uuid::new_v4().as_bytes())
}

/// Renders an id in the wire format: 32 characters of zero-padded lowercase hex.
pub fn format_id(id: Id) -> String {
    format!("{id:0>32x}")
}

/// Parses an id from the wire format, tolerating the dashes that appear in the
/// GUIDs an identity provider hands us.
pub fn parse_id(value: &str) -> Option<Id> {
    u128::from_str_radix(value.replace('-', "").as_str(), 16).ok()
}

/// Parses an id, reporting a 400 that names the field when it doesn't parse.
pub fn parse_id_or_400(value: &str, description: &str) -> Result<Id, ApiError> {
    parse_id(value).ok_or_else(|| {
        ApiError::bad_request(&format!(
            "The {description} you provided could not be parsed. Please check it and try again."
        ))
    })
}

/// The MD5 of a lowercased, trimmed email address.
///
/// This is a *pseudonym*, not an anonymisation: MD5 over a low-entropy input is
/// trivially reversible by dictionary attack. It exists because it is both the
/// existing wire contract (`GET /api/v3/user/{hash}`) and the Gravatar key, and
/// no authorization decision is ever derived from it.
pub fn email_hash(email: &str) -> Id {
    u128::from_be_bytes(md5::compute(email.to_lowercase().trim().as_bytes()).into())
}
