use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::controls::{Button, Tag, TextInput};

#[derive(Properties, PartialEq)]
pub struct TagEditorProps {
    pub tags: Vec<String>,
    pub onchange: Callback<Vec<String>>,
}

#[function_component(TagEditor)]
pub fn tag_editor(props: &TagEditorProps) -> Html {
    let editing = use_state(|| false);
    let draft = use_state(String::new);
    let input_ref = use_node_ref();

    // Revealing the field is only half of it: without this the caret stays
    // wherever it was, and what the user types lands in the previous field.
    {
        let input_ref = input_ref.clone();

        use_effect_with(*editing, move |editing| {
            if *editing
                && let Some(input) = input_ref
                    .get()
                    .and_then(|node| node.dyn_into::<HtmlInputElement>().ok())
            {
                let _ = input.focus();
            }
        });
    }

    let confirm = {
        let editing = editing.clone();
        let draft = draft.clone();
        let tags = props.tags.clone();
        let onchange = props.onchange.clone();

        Callback::from(move |_: ()| {
            let value = draft.trim().to_string();

            if !value.is_empty() && !tags.contains(&value) {
                let mut updated = tags.clone();
                updated.push(value);
                onchange.emit(updated);
            }

            draft.set(String::new());
            editing.set(false);
        })
    };

    let remove = {
        let tags = props.tags.clone();
        let onchange = props.onchange.clone();

        Callback::from(move |tag: String| {
            onchange.emit(tags.iter().filter(|t| **t != tag).cloned().collect());
        })
    };

    html! {
        <div class="tag-editor">
            { for props.tags.iter().map(|tag| {
                let tag = tag.clone();
                let remove = remove.clone();

                html! {
                    <Tag
                        key={tag.clone()}
                        onclose={Callback::from({
                            let tag = tag.clone();
                            move |_| remove.emit(tag.clone())
                        })}>
                        { Html::from(tag.clone()) }
                    </Tag>
                }
            }) }

            if *editing {
                <span class="tag-editor__input">
                    <TextInput
                        node_ref={input_ref.clone()}
                        value={(*draft).clone()}
                        placeholder="Tag"
                        onchange={{
                            let draft = draft.clone();
                            Callback::from(move |value| draft.set(value))
                        }}
                        onsubmit={confirm.clone()}
                        onblur={confirm} />
                </span>
            } else {
                <Button
                    small={true}
                    onclick={{
                        let editing = editing.clone();
                        Callback::from(move |_: MouseEvent| editing.set(true))
                    }}>
                    { "+ New Tag" }
                </Button>
            }
        </div>
    }
}
