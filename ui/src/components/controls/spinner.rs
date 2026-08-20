use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SpinnerProps {
    #[prop_or_default]
    pub label: AttrValue,
}

#[function_component(Spinner)]
pub fn spinner(props: &SpinnerProps) -> Html {
    html! {
        <div class="spinner" role="status">
            <span class="spinner__ring" aria-hidden="true"></span>
            if !props.label.is_empty() {
                <span class="spinner__label">{ &props.label }</span>
            }
        </div>
    }
}
