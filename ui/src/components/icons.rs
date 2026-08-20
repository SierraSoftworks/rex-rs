use yew::prelude::*;

/// The icon set, drawn inline.
///
/// These replace the icon package the old interface pulled in for a dozen
/// glyphs. Inline SVG keeps them in the wasm bundle, so there is no second
/// request and nothing to 404.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Plus,
    Check,
    Undo,
    Delete,
    ArrowRight,
    Document,
    DocumentAdd,
    DocumentCopy,
    DocumentChecked,
    SignOut,
}

impl Icon {
    fn path(&self) -> &'static str {
        match self {
            Icon::Plus => "M12 5v14M5 12h14",
            Icon::Check => "M4 12.5l5 5L20 6.5",
            Icon::Undo => "M4 9h11a5 5 0 010 10h-6M4 9l4-4M4 9l4 4",
            Icon::Delete => "M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13M10 11v6M14 11v6",
            Icon::ArrowRight => "M4 12h15M13 6l6 6-6 6",
            Icon::Document => "M6 3h8l4 4v14H6zM14 3v4h4",
            Icon::DocumentAdd => "M6 3h8l4 4v14H6zM14 3v4h4M12 11v6M9 14h6",
            Icon::DocumentCopy => "M9 3h6l3 3v11H9zM15 3v3h3M15 17v4H6V8h3",
            Icon::DocumentChecked => "M6 3h8l4 4v14H6zM14 3v4h4M9 14l2.5 2.5L16 12",
            Icon::SignOut => "M15 4h4v16h-4M11 8l-4 4 4 4M7 12h10",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct IconProps {
    pub icon: Icon,
    #[prop_or(16)]
    pub size: u32,
}

#[function_component(IconView)]
pub fn icon_view(props: &IconProps) -> Html {
    let size = props.size.to_string();

    html! {
        <svg
            class="icon"
            width={size.clone()}
            height={size}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true">
            <path d={props.icon.path()} />
        </svg>
    }
}
