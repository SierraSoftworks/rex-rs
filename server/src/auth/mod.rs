//! Authentication: standard OIDC, configured rather than hard-coded.
//!
//! The browser runs the authorization code flow in a popup, the server performs
//! the confidential code exchange with its client secret, and the resulting
//! **ID token** is the bearer that Rex's API accepts. No cookies are involved,
//! so there is no CSRF surface to defend.

mod acl;
mod oidc;
mod token;

pub use acl::{Acl, AclContext};
pub use oidc::{OidcState, TokenSet};
pub use token::AuthToken;

#[cfg(test)]
mod tests;
