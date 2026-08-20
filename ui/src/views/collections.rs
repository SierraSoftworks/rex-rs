use rex_api::CollectionV3;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    app::Route,
    components::{
        Icon, Protected,
        controls::{Button, ButtonGroup, ButtonKind, Spinner},
        use_notifier,
    },
};

#[function_component(Collections)]
pub fn collections() -> Html {
    html! {
        <Protected>
            <CollectionsList />
        </Protected>
    }
}

#[function_component(CollectionsList)]
fn collections_list() -> Html {
    let notifier = use_notifier();
    let navigator = use_navigator().expect("a router");
    let collections = use_state(Vec::<CollectionV3>::new);
    let loading = use_state(|| true);

    {
        let collections = collections.clone();
        let loading = loading.clone();
        let notifier = notifier.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::collections().await {
                    Ok(loaded) => collections.set(loaded),
                    // A caller with nothing yet is not an error worth a toast.
                    Err(err) if err.code == 404 => collections.set(Vec::new()),
                    Err(err) => notifier.error(&err),
                }

                loading.set(false);
            });
        });
    }

    let go = {
        let navigator = navigator.clone();
        move |route: Route| {
            let navigator = navigator.clone();
            Callback::from(move |_: MouseEvent| navigator.push(&route))
        }
    };

    html! {
        <section class="collections">
            <h1>{ "Collections" }</h1>
            <p>{ "Collections allow you to organize and share your ideas with others." }</p>

            if *loading {
                <Spinner label="Loading your collections" />
            } else if collections.is_empty() {
                <p class="muted">{ "You don't have any collections yet." }</p>
            }

            { for collections.iter().map(|collection| {
                let cid = collection.id.clone().unwrap_or_default();

                html! {
                    <div class="collections__item" key={cid.clone()}>
                        <h4>{ &collection.name }</h4>

                        <ButtonGroup>
                            <Button
                                kind={ButtonKind::Primary}
                                icon={Icon::Document}
                                onclick={go(Route::Collection { cid: cid.clone() })}>
                                { "Open" }
                            </Button>
                            <Button
                                icon={Icon::DocumentAdd}
                                onclick={go(Route::NewIdea { cid: cid.clone() })}>
                                { "New Idea" }
                            </Button>
                            <Button
                                icon={Icon::DocumentChecked}
                                onclick={go(Route::Manage { cid: cid.clone() })}>
                                { "Manage" }
                            </Button>
                        </ButtonGroup>
                    </div>
                }
            }) }
        </section>
    }
}
