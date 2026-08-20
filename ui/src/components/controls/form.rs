use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FormProps {
    #[prop_or_default]
    pub onsubmit: Callback<()>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Form)]
pub fn form(props: &FormProps) -> Html {
    let onsubmit = {
        let callback = props.onsubmit.clone();
        Callback::from(move |event: SubmitEvent| {
            // The form posts through the API client, not the browser.
            event.prevent_default();
            callback.emit(());
        })
    };

    html! {
        <form class="form" {onsubmit}>{ for props.children.iter() }</form>
    }
}

#[derive(Properties, PartialEq)]
pub struct FormFieldProps {
    pub label: AttrValue,
    /// Shown under the field, for validation and lookup feedback.
    #[prop_or_default]
    pub hint: Option<AttrValue>,
    /// Set when the field holds a composite control -- several elements working
    /// together, like the tag editor -- rather than one input.
    ///
    /// A `<label>` attaches itself to the first labelable thing inside it, so
    /// wrapping a composite control names its first button after the field.
    /// A labelled group says what is actually true.
    #[prop_or_default]
    pub group: bool,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(FormField)]
pub fn form_field(props: &FormFieldProps) -> Html {
    let body = html! {
        <>
            <span class="form-field__label">{ &props.label }</span>
            <span class="form-field__control">{ for props.children.iter() }</span>
            if let Some(hint) = props.hint.clone() {
                <span class="form-field__hint">{ hint }</span>
            }
        </>
    };

    if props.group {
        html! {
            <div class="form-field" role="group" aria-label={props.label.clone()}>
                { body }
            </div>
        }
    } else {
        html! { <label class="form-field">{ body }</label> }
    }
}
