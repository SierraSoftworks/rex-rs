//! The pieces the views are built from.
//!
//! `controls` is Rex's own control library, which replaces the component
//! library the previous interface leaned on. Everything else here is specific
//! to Rex: the header, the idea display and its controls, the tag editor, and
//! the toast host.

pub mod controls;

mod header;
mod icons;
mod idea_controls;
mod idea_display;
mod markdown_view;
mod notifications;
mod protected;
mod tag_editor;

pub use header::Header;
pub use icons::Icon;
pub use idea_controls::IdeaControls;
pub use idea_display::IdeaDisplay;
pub use markdown_view::MarkdownView;
pub use notifications::{NotificationProvider, use_notifier};
pub use protected::Protected;
pub use tag_editor::TagEditor;

/// The Gravatar URL for an email hash, at the requested size.
///
/// Gravatar is part of how Rex looks, so it stays; the stock fallback avatar it
/// used to point at a third-party CDN for is now drawn locally.
pub fn gravatar(email_hash: &str, size: u32) -> String {
    format!("https://www.gravatar.com/avatar/{email_hash}?s={size}&d=mp")
}
