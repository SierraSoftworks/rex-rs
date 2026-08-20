use rex_api::IdeaV3;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    app::Route,
    components::{IdeaControls, IdeaDisplay, Protected, controls::Spinner, use_notifier},
};

#[derive(Properties, PartialEq)]
pub struct IdeaProps {
    pub collection: String,
    pub idea: String,
}

/// One idea, addressed directly.
#[function_component(Idea)]
pub fn idea(props: &IdeaProps) -> Html {
    html! {
        <Protected>
            <IdeaContent collection={props.collection.clone()} idea={props.idea.clone()} />
        </Protected>
    }
}

#[function_component(IdeaContent)]
fn idea_content(props: &IdeaProps) -> Html {
    let notifier = use_notifier();
    let navigator = use_navigator().expect("a router");
    let idea = use_state(|| None::<IdeaV3>);
    let busy = use_state(|| true);

    {
        let idea = idea.clone();
        let busy = busy.clone();
        let notifier = notifier.clone();
        let collection = props.collection.clone();
        let id = props.idea.clone();

        use_effect_with((collection.clone(), id.clone()), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::idea(&collection, &id).await {
                    Ok(loaded) => idea.set(Some(loaded)),
                    Err(err) => notifier.error(&err),
                }

                busy.set(false);
            });
        });
    }

    let on_complete = {
        let idea = idea.clone();
        let busy = busy.clone();
        let notifier = notifier.clone();

        Callback::from(move |completed: bool| {
            let Some(current) = (*idea).clone() else {
                return;
            };

            let idea = idea.clone();
            let busy = busy.clone();
            let notifier = notifier.clone();

            busy.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                let updated = IdeaV3 {
                    completed: Some(completed),
                    ..current
                };

                match api::store_idea(&updated).await {
                    Ok(saved) => idea.set(Some(saved)),
                    Err(err) => notifier.error(&err),
                }

                busy.set(false);
            });
        })
    };

    // The old interface rendered a delete button here that could never be
    // reached, because the controls were told not to allow deleting.
    let on_delete = {
        let idea = idea.clone();
        let busy = busy.clone();
        let notifier = notifier.clone();
        let navigator = navigator.clone();

        Callback::from(move |_: ()| {
            let Some(current) = (*idea).clone() else {
                return;
            };

            let busy = busy.clone();
            let notifier = notifier.clone();
            let navigator = navigator.clone();

            busy.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                let collection = current.collection.clone().unwrap_or_default();
                let id = current.id.clone().unwrap_or_default();

                match api::remove_idea(&collection, &id).await {
                    Ok(()) => navigator.push(&Route::Collection { cid: collection }),
                    Err(err) => {
                        notifier.error(&err);
                        busy.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="fill">
            <div class="center">
                if let Some(current) = (*idea).clone() {
                    <IdeaDisplay idea={current} />
                } else if *busy {
                    <Spinner label="Loading" />
                } else {
                    <p class="muted">
                        { "Oops, it looks like the idea you're looking for could not be found." }
                    </p>
                }
            </div>

            if let Some(current) = (*idea).clone() {
                <div class="bottom">
                    <IdeaControls
                        idea={current}
                        allow_complete={true}
                        allow_delete={true}
                        busy={*busy}
                        on_complete={on_complete}
                        on_delete={on_delete} />
                </div>
            }
        </div>
    }
}
