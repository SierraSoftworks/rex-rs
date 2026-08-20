//! The REST contract shared between the Rex server and its web UI.
//!
//! Everything in here is pure serde: no web framework, no database, no runtime.
//! That keeps the crate compilable for `wasm32-unknown-unknown` (where the UI
//! consumes it) as well as for the server's native target. The only exception
//! is the optional `actix` feature, which adds an actix-web `ResponseError`
//! implementation for [`ApiError`] so that the server can return the shared
//! wire type straight out of its handlers rather than duplicating it.

mod auth;
mod collection;
mod error;
mod health;
mod idea;
mod role_assignment;
mod user;

pub use auth::*;
pub use collection::*;
pub use error::*;
pub use health::*;
pub use idea::*;
pub use role_assignment::*;
pub use user::*;
