use rex_api::IdeaV3;
use yew::prelude::*;

use super::{
    MarkdownView,
    controls::{Tag, TagKind},
};

#[derive(Properties, PartialEq)]
pub struct IdeaDisplayProps {
    pub idea: IdeaV3,
}

/// One idea, presented the way Rex has always presented it: a thin-weight title
/// with a hanging quotation mark in the left margin, and the description below.
#[function_component(IdeaDisplay)]
pub fn idea_display(props: &IdeaDisplayProps) -> Html {
    let idea = &props.idea;

    let mut tags: Vec<String> = idea.tags.clone().unwrap_or_default().into_iter().collect();
    // A HashSet has no order of its own, and a title whose tags rearrange
    // themselves between renders looks broken.
    tags.sort();

    html! {
        <div class="idea">
            <h2 class="idea__title">
                { &idea.name }

                if idea.completed.unwrap_or(false) {
                    <Tag kind={TagKind::Success}>{ "Done" }</Tag>
                }

                { for tags.into_iter().map(|tag| html! { <Tag key={tag.clone()}>{ Html::from(tag.clone()) }</Tag> }) }
            </h2>

            <MarkdownView
                class={classes!("idea__description", "markdown")}
                value={idea.description.clone()} />
        </div>
    }
}
