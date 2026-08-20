use rex_api::IdeaV3;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    app::Route,
    components::{
        Icon, Protected, TagEditor,
        controls::{Button, ButtonKind, Form, FormField, TextArea, TextInput},
        use_notifier,
    },
};

#[derive(Properties, PartialEq)]
pub struct NewIdeaProps {
    pub collection: String,
}

#[function_component(NewIdea)]
pub fn new_idea(props: &NewIdeaProps) -> Html {
    html! {
        <Protected>
            <NewIdeaForm collection={props.collection.clone()} />
        </Protected>
    }
}

#[function_component(NewIdeaForm)]
fn new_idea_form(props: &NewIdeaProps) -> Html {
    let notifier = use_notifier();
    let navigator = use_navigator().expect("a router");

    let name = use_state(String::new);
    let description = use_state(String::new);
    let tags = use_state(Vec::<String>::new);
    let saving = use_state(|| false);

    let onsubmit = {
        let notifier = notifier.clone();
        let navigator = navigator.clone();
        let collection = props.collection.clone();
        let name = name.clone();
        let description = description.clone();
        let tags = tags.clone();
        let saving = saving.clone();

        Callback::from(move |_: ()| {
            if name.trim().is_empty() {
                notifier.message("Nearly", "An idea needs a name before it can be saved.");
                return;
            }

            let notifier = notifier.clone();
            let navigator = navigator.clone();
            let collection = collection.clone();
            let saving = saving.clone();

            let idea = IdeaV3 {
                id: None,
                collection: None,
                name: name.trim().to_string(),
                description: (*description).clone(),
                tags: if tags.is_empty() {
                    None
                } else {
                    Some(tags.iter().cloned().collect())
                },
                completed: None,
            };

            saving.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                match api::new_idea(Some(&collection), &idea).await {
                    Ok(created) => navigator.push(&Route::Idea {
                        cid: created.collection.unwrap_or(collection),
                        iid: created.id.unwrap_or_default(),
                    }),
                    Err(err) => {
                        notifier.error(&err);
                        saving.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="fill">
            <div class="center">
                <h2 class="new-idea__title">{ "Add a new idea" }</h2>

                <Form onsubmit={onsubmit.clone()}>
                    <FormField label="Name">
                        <TextInput
                            value={(*name).clone()}
                            placeholder="Give your idea a name"
                            onchange={{
                                let name = name.clone();
                                Callback::from(move |value| name.set(value))
                            }} />
                    </FormField>

                    <FormField label="Description">
                        <TextArea
                            value={(*description).clone()}
                            placeholder="Describe the idea"
                            onchange={{
                                let description = description.clone();
                                Callback::from(move |value| description.set(value))
                            }} />
                    </FormField>

                    <FormField label="Tags" group={true}>
                        <TagEditor
                            tags={(*tags).clone()}
                            onchange={{
                                let tags = tags.clone();
                                Callback::from(move |updated| tags.set(updated))
                            }} />
                    </FormField>

                    <div>
                        <Button
                            kind={ButtonKind::Primary}
                            icon={Icon::Plus}
                            disabled={*saving}
                            onclick={Callback::from(move |_: MouseEvent| onsubmit.emit(()))}>
                            { "Save" }
                        </Button>
                    </div>
                </Form>
            </div>
        </div>
    }
}
