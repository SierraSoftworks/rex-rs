use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SelectProps {
    pub value: AttrValue,
    pub options: Vec<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onchange: Callback<String>,
}

#[function_component(Select)]
pub fn select(props: &SelectProps) -> Html {
    let onchange = {
        let callback = props.onchange.clone();
        Callback::from(move |event: Event| {
            let value = event
                .target_dyn_into::<web_sys::HtmlSelectElement>()
                .map(|select| select.value())
                .unwrap_or_default();

            callback.emit(value);
        })
    };

    html! {
        <select class="select" disabled={props.disabled} {onchange}>
            { for props.options.iter().map(|option| html! {
                <option
                    value={option.clone()}
                    selected={option == &props.value}>
                    { option }
                </option>
            }) }
        </select>
    }
}
