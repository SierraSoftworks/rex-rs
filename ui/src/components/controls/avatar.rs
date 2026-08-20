use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AvatarProps {
    /// Absent while we do not know who somebody is, which draws the placeholder.
    #[prop_or_default]
    pub src: Option<AttrValue>,
    #[prop_or(32)]
    pub size: u32,
    #[prop_or_default]
    pub alt: AttrValue,
}

#[function_component(Avatar)]
pub fn avatar(props: &AvatarProps) -> Html {
    let size = format!("{}px", props.size);
    let style = format!("width: {size}; height: {size};");

    match props.src.clone() {
        Some(src) => html! {
            <img class="avatar" {style} {src} alt={props.alt.clone()} />
        },
        // Drawn locally rather than fetched from a third-party CDN, which is
        // where the previous placeholder lived.
        None => html! {
            <span class="avatar avatar--placeholder" {style} role="img" aria-label={props.alt.clone()}>
                <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                    <path d="M12 12a4 4 0 100-8 4 4 0 000 8zm0 2c-4 0-7 2-7 4.5V20h14v-1.5c0-2.5-3-4.5-7-4.5z" />
                </svg>
            </span>
        },
    }
}
