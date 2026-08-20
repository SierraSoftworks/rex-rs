use yew::prelude::*;

use crate::auth::input_value;

#[derive(Properties, PartialEq)]
pub struct TextInputProps {
    pub value: AttrValue,
    /// Handed out so a caller can focus the field once it appears.
    #[prop_or_default]
    pub node_ref: NodeRef,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or("text".into())]
    pub input_type: AttrValue,
    #[prop_or_default]
    pub onchange: Callback<String>,
    /// Fired when the user presses Enter, for the "type and confirm" pattern.
    #[prop_or_default]
    pub onsubmit: Callback<()>,
    #[prop_or_default]
    pub onblur: Callback<()>,
}

#[function_component(TextInput)]
pub fn text_input(props: &TextInputProps) -> Html {
    let oninput = {
        let onchange = props.onchange.clone();
        Callback::from(move |event: InputEvent| onchange.emit(input_value(&event.into())))
    };

    let onkeydown = {
        let onsubmit = props.onsubmit.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                event.prevent_default();
                onsubmit.emit(());
            }
        })
    };

    let onblur = {
        let callback = props.onblur.clone();
        Callback::from(move |_: FocusEvent| callback.emit(()))
    };

    html! {
        <input
            ref={props.node_ref.clone()}
            class="text-input"
            type={props.input_type.clone()}
            value={props.value.clone()}
            placeholder={props.placeholder.clone()}
            disabled={props.disabled}
            {oninput}
            {onkeydown}
            {onblur} />
    }
}

#[derive(Properties, PartialEq)]
pub struct TextAreaProps {
    pub value: AttrValue,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or(4)]
    pub rows: u32,
    #[prop_or_default]
    pub onchange: Callback<String>,
}

#[function_component(TextArea)]
pub fn text_area(props: &TextAreaProps) -> Html {
    let oninput = {
        let onchange = props.onchange.clone();
        Callback::from(move |event: InputEvent| {
            let value = event
                .target_dyn_into::<web_sys::HtmlTextAreaElement>()
                .map(|area| area.value())
                .unwrap_or_default();

            onchange.emit(value);
        })
    };

    html! {
        <textarea
            class="text-input text-input--multiline"
            rows={props.rows.to_string()}
            placeholder={props.placeholder.clone()}
            value={props.value.clone()}
            {oninput} />
    }
}
