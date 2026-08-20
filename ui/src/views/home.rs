use rex_api::IdeaV3;
use yew::prelude::*;

use crate::{
    api,
    components::{IdeaControls, IdeaDisplay, Protected, controls::Spinner, use_notifier},
};

#[derive(Properties, PartialEq)]
pub struct HomeProps {
    /// `None` means the caller's default collection.
    #[prop_or_default]
    pub collection: Option<String>,
}

/// The core loop: one random idea, and four things you can do about it.
#[function_component(Home)]
pub fn home(props: &HomeProps) -> Html {
    html! {
        <Protected>
            <HomeContent collection={props.collection.clone()} />
        </Protected>
    }
}

#[function_component(HomeContent)]
fn home_content(props: &HomeProps) -> Html {
    let notifier = use_notifier();
    let idea = use_state(|| None::<IdeaV3>);
    let busy = use_state(|| true);

    let load = {
        let idea = idea.clone();
        let busy = busy.clone();
        let notifier = notifier.clone();
        let collection = props.collection.clone();

        Callback::from(move |_: ()| {
            let idea = idea.clone();
            let busy = busy.clone();
            let notifier = notifier.clone();
            let collection = collection.clone();

            busy.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                match api::random_idea(collection.as_deref()).await {
                    Ok(next) => idea.set(Some(next)),
                    Err(err) if err.code == 404 => idea.set(None),
                    Err(err) => {
                        notifier.error(&err);
                        idea.set(None);
                    }
                }

                busy.set(false);
            });
        })
    };

    {
        let load = load.clone();
        use_effect_with(props.collection.clone(), move |_| load.emit(()));
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

    let on_delete = {
        let idea = idea.clone();
        let busy = busy.clone();
        let notifier = notifier.clone();
        let load = load.clone();

        Callback::from(move |_: ()| {
            let Some(current) = (*idea).clone() else {
                return;
            };

            let busy = busy.clone();
            let notifier = notifier.clone();
            let load = load.clone();

            busy.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                let collection = current.collection.clone().unwrap_or_default();
                let id = current.id.clone().unwrap_or_default();

                match api::remove_idea(&collection, &id).await {
                    Ok(()) => load.emit(()),
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
                    <Spinner label="Finding you something to do" />
                } else {
                    <p class="home__empty">
                        { "There's nothing here yet. Add an idea and it'll show up." }
                    </p>
                }
            </div>

            if let Some(current) = (*idea).clone() {
                <div class="bottom">
                    <IdeaControls
                        idea={current}
                        allow_complete={true}
                        allow_next={true}
                        allow_delete={true}
                        busy={*busy}
                        on_complete={on_complete}
                        on_next={load.clone()}
                        on_delete={on_delete} />
                </div>
            }
        </div>
    }
}
