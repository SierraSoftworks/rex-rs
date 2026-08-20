use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    app::Route,
    components::{
        Icon, Protected,
        controls::{Button, ButtonKind, Form, FormField, TextInput},
        use_notifier,
    },
};

#[function_component(NewCollection)]
pub fn new_collection() -> Html {
    html! {
        <Protected>
            <NewCollectionForm />
        </Protected>
    }
}

#[function_component(NewCollectionForm)]
fn new_collection_form() -> Html {
    let notifier = use_notifier();
    let navigator = use_navigator().expect("a router");

    let name = use_state(String::new);
    let saving = use_state(|| false);

    let onsubmit = {
        let notifier = notifier.clone();
        let navigator = navigator.clone();
        let name = name.clone();
        let saving = saving.clone();

        Callback::from(move |_: ()| {
            if name.trim().is_empty() {
                notifier.message(
                    "Nearly",
                    "A collection needs a name before it can be saved.",
                );
                return;
            }

            let notifier = notifier.clone();
            let navigator = navigator.clone();
            let saving = saving.clone();
            let name = name.trim().to_string();

            saving.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                match api::new_collection(&name).await {
                    Ok(created) => navigator.push(&Route::Manage {
                        cid: created.id.unwrap_or_default(),
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
                <Form onsubmit={onsubmit.clone()}>
                    <FormField label="Name">
                        <TextInput
                            value={(*name).clone()}
                            placeholder="Give your collection a name"
                            onchange={{
                                let name = name.clone();
                                Callback::from(move |value| name.set(value))
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
