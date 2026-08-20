use rex_api::UserV3;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    app::Route,
    components::{
        Protected,
        controls::{Avatar, Button, ButtonKind, Form, FormField, Select, Spinner, TextInput},
        gravatar, use_notifier,
    },
};

/// How long to wait after the last keystroke before looking somebody up.
const DEBOUNCE_MS: u32 = 500;

const ROLES: [&str; 3] = ["Owner", "Contributor", "Viewer"];

#[derive(Properties, PartialEq)]
pub struct InviteProps {
    pub collection: String,
}

#[function_component(Invite)]
pub fn invite(props: &InviteProps) -> Html {
    html! {
        <Protected>
            <InviteForm collection={props.collection.clone()} />
        </Protected>
    }
}

/// What the email box currently resolves to.
#[derive(Clone, PartialEq)]
enum Lookup {
    Empty,
    Searching,
    Found(UserV3),
    /// The previous interface tracked this state and then never showed it.
    NotFound,
}

#[function_component(InviteForm)]
fn invite_form(props: &InviteProps) -> Html {
    let notifier = use_notifier();
    let navigator = use_navigator().expect("a router");

    let email = use_state(String::new);
    let role = use_state(|| String::from("Viewer"));
    let lookup = use_state(|| Lookup::Empty);
    let saving = use_state(|| false);

    // Look the address up once typing settles, so a twelve-character address is
    // one request rather than twelve.
    {
        let email = email.clone();
        let lookup = lookup.clone();

        use_effect_with((*email).clone(), move |address| {
            let address = address.trim().to_string();

            if address.is_empty() {
                lookup.set(Lookup::Empty);
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }

            lookup.set(Lookup::Searching);

            let cancelled = std::rc::Rc::new(std::cell::Cell::new(false));

            {
                let cancelled = cancelled.clone();
                let lookup = lookup.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(DEBOUNCE_MS).await;

                    if cancelled.get() {
                        return;
                    }

                    let result = api::user_by_email(&address).await;

                    if cancelled.get() {
                        return;
                    }

                    lookup.set(match result {
                        Ok(user) => Lookup::Found(user),
                        Err(_) => Lookup::NotFound,
                    });
                });
            }

            Box::new(move || cancelled.set(true)) as Box<dyn FnOnce()>
        });
    }

    let onsubmit = {
        let notifier = notifier.clone();
        let navigator = navigator.clone();
        let collection = props.collection.clone();
        let lookup = lookup.clone();
        let role = role.clone();
        let saving = saving.clone();

        Callback::from(move |_: ()| {
            let Lookup::Found(user) = (*lookup).clone() else {
                notifier.message(
                    "Nearly",
                    "We need to find somebody by their email address before we can invite them.",
                );
                return;
            };

            let notifier = notifier.clone();
            let navigator = navigator.clone();
            let collection = collection.clone();
            let role = (*role).clone();
            let saving = saving.clone();

            saving.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                match api::set_role_assignment(&collection, &user.id, &role).await {
                    Ok(_) => navigator.push(&Route::Manage { cid: collection }),
                    Err(err) => {
                        notifier.error(&err);
                        saving.set(false);
                    }
                }
            });
        })
    };

    let (avatar, name, hint) = match &*lookup {
        Lookup::Empty => (None, String::from("Someone"), None),
        Lookup::Searching => (None, String::from("Someone"), None),
        Lookup::Found(user) => (
            Some(gravatar(&user.email_hash, 50)),
            user.first_name.clone(),
            None,
        ),
        Lookup::NotFound => (
            None,
            String::from("Someone"),
            Some(
                "Nobody with that address has used Rex yet. They'll need to sign in once before you can share with them.",
            ),
        ),
    };

    html! {
        <div class="fill">
            <div class="center invite">
                <h1 class="invite__heading">
                    <Avatar size={50} src={avatar} alt={name.clone()} />
                    { format!("Invite {name}") }
                </h1>

                <Form onsubmit={onsubmit.clone()}>
                    <FormField label="Role">
                        <Select
                            value={(*role).clone()}
                            options={ROLES.iter().map(|r| AttrValue::from(*r)).collect::<Vec<_>>()}
                            onchange={{
                                let role = role.clone();
                                Callback::from(move |value| role.set(value))
                            }} />
                    </FormField>

                    <FormField label="Email Address" hint={hint.map(AttrValue::from)}>
                        <TextInput
                            input_type="email"
                            value={(*email).clone()}
                            placeholder="someone@example.com"
                            onchange={{
                                let email = email.clone();
                                Callback::from(move |value| email.set(value))
                            }} />
                    </FormField>

                    <div class="invite__actions">
                        if matches!(*lookup, Lookup::Searching) {
                            <Spinner label="Looking them up" />
                        }

                        <Button
                            kind={ButtonKind::Primary}
                            disabled={*saving || !matches!(*lookup, Lookup::Found(_))}
                            onclick={Callback::from(move |_: MouseEvent| onsubmit.emit(()))}>
                            { "Invite" }
                        </Button>
                    </div>
                </Form>
            </div>
        </div>
    }
}
