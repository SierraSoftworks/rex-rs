use rex_api::IdeaV3;
use yew::prelude::*;

use super::{Icon, controls::Button};

#[derive(Properties, PartialEq)]
pub struct IdeaControlsProps {
    pub idea: IdeaV3,
    #[prop_or_default]
    pub allow_complete: bool,
    #[prop_or_default]
    pub allow_next: bool,
    #[prop_or_default]
    pub allow_delete: bool,
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub on_complete: Callback<bool>,
    #[prop_or_default]
    pub on_next: Callback<()>,
    #[prop_or_default]
    pub on_delete: Callback<()>,
}

/// The core loop's controls: done, undo, delete, next.
#[function_component(IdeaControls)]
pub fn idea_controls(props: &IdeaControlsProps) -> Html {
    let completed = props.idea.completed.unwrap_or(false);

    let on_done = {
        let callback = props.on_complete.clone();
        Callback::from(move |_: MouseEvent| callback.emit(true))
    };

    let on_undo = {
        let callback = props.on_complete.clone();
        Callback::from(move |_: MouseEvent| callback.emit(false))
    };

    let on_next = {
        let callback = props.on_next.clone();
        Callback::from(move |_: MouseEvent| callback.emit(()))
    };

    let on_delete = {
        let callback = props.on_delete.clone();
        Callback::from(move |_: MouseEvent| callback.emit(()))
    };

    html! {
        <p class="idea-controls">
            if props.allow_complete && !completed {
                <Button icon={Icon::Check} disabled={props.busy} onclick={on_done}>
                    { "Mark as Done" }
                </Button>
            }

            if props.allow_complete && completed {
                <Button icon={Icon::Undo} disabled={props.busy} onclick={on_undo}>
                    { "Undo" }
                </Button>
            }

            if props.allow_delete {
                <Button icon={Icon::Delete} disabled={props.busy} onclick={on_delete}>
                    { "Delete" }
                </Button>
            }

            if props.allow_next {
                <Button icon={Icon::ArrowRight} disabled={props.busy} onclick={on_next}>
                    { "Next" }
                </Button>
            }
        </p>
    }
}
