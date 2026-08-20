use yew::prelude::*;

/// A hover label.
///
/// CSS-only, because a tooltip that needs JavaScript to appear is a tooltip
/// that sometimes does not.
#[derive(Properties, PartialEq)]
pub struct TooltipProps {
    pub content: AttrValue,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Tooltip)]
pub fn tooltip(props: &TooltipProps) -> Html {
    html! {
        <span class="tooltip">
            { for props.children.iter() }
            <span class="tooltip__content" role="tooltip">{ &props.content }</span>
        </span>
    }
}
