use yew::prelude::*;

use crate::components::{Icon, icons::IconView};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonKind {
    #[default]
    Default,
    Primary,
    Danger,
}

impl ButtonKind {
    fn class(&self) -> &'static str {
        match self {
            ButtonKind::Default => "button--default",
            ButtonKind::Primary => "button--primary",
            ButtonKind::Danger => "button--danger",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    #[prop_or_default]
    pub kind: ButtonKind,
    #[prop_or_default]
    pub icon: Option<Icon>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub small: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    let mut classes = classes!("button", props.kind.class());
    if props.small {
        classes.push("button--small");
    }

    html! {
        <button
            type="button"
            class={classes}
            disabled={props.disabled}
            onclick={props.onclick.clone()}>
            if let Some(icon) = props.icon {
                <IconView icon={icon} />
            }
            if !props.children.is_empty() {
                <span class="button__label">{ for props.children.iter() }</span>
            }
        </button>
    }
}

/// A button with no label, for table rows and other tight spaces.
#[derive(Properties, PartialEq)]
pub struct IconButtonProps {
    pub icon: Icon,
    /// Announced to screen readers, since there is no visible text.
    pub label: AttrValue,
    #[prop_or_default]
    pub kind: ButtonKind,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

#[function_component(IconButton)]
pub fn icon_button(props: &IconButtonProps) -> Html {
    html! {
        <button
            type="button"
            class={classes!("button", "button--icon", props.kind.class())}
            aria-label={props.label.clone()}
            title={props.label.clone()}
            disabled={props.disabled}
            onclick={props.onclick.clone()}>
            <IconView icon={props.icon} />
        </button>
    }
}

/// Joins buttons into one continuous control.
#[derive(Properties, PartialEq)]
pub struct ButtonGroupProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ButtonGroup)]
pub fn button_group(props: &ButtonGroupProps) -> Html {
    html! {
        <div class="button-group">{ for props.children.iter() }</div>
    }
}
