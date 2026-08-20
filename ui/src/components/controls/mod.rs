//! Rex's own control library.
//!
//! These replace the component library the previous interface used. Each one is
//! deliberately plain: thin weights, hairline borders, generous whitespace, and
//! the same blue the app has always used for its primary action, so the rebuilt
//! interface still reads as Rex.
//!
//! Every control has a specimen in the gallery at `/demo/controls` (debug
//! builds only). Add or change a control, add or change its specimen.

mod avatar;
mod button;
mod form;
mod select;
mod spinner;
mod table;
mod tag;
mod text_input;
mod tooltip;

pub use avatar::Avatar;
pub use button::{Button, ButtonGroup, ButtonKind, IconButton};
pub use form::{Form, FormField};
pub use select::Select;
pub use spinner::Spinner;
pub use table::{Table, TableColumn};
pub use tag::{Tag, TagKind};
pub use text_input::{TextArea, TextInput};
pub use tooltip::Tooltip;
