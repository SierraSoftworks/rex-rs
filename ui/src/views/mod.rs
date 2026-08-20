//! One module per screen, each with its stylesheet beside it.

mod callback;
mod collections;
mod home;
mod idea;
mod invite;
mod manage;
mod new_collection;
mod new_idea;

#[cfg(debug_assertions)]
mod gallery;

pub use callback::Callback;
pub use collections::Collections;
pub use home::Home;
pub use idea::Idea;
pub use invite::Invite;
pub use manage::Manage;
pub use new_collection::NewCollection;
pub use new_idea::NewIdea;

#[cfg(debug_assertions)]
pub use gallery::Gallery;
