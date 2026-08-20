use yew::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagKind {
    #[default]
    Default,
    Success,
}

#[derive(Properties, PartialEq)]
pub struct TagProps {
    #[prop_or_default]
    pub kind: TagKind,
    /// When set, the tag shows a dismiss affordance and fires this on click.
    #[prop_or_default]
    pub onclose: Option<Callback<()>>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Tag)]
pub fn tag(props: &TagProps) -> Html {
    let kind = match props.kind {
        TagKind::Default => "tag--default",
        TagKind::Success => "tag--success",
    };

    html! {
        <span class={classes!("tag", kind)}>
            { for props.children.iter() }

            if let Some(onclose) = props.onclose.clone() {
                <button
                    type="button"
                    class="tag__close"
                    aria-label="Remove"
                    onclick={Callback::from(move |_: MouseEvent| onclose.emit(()))}>
                    { "×" }
                </button>
            }
        </span>
    }
}
