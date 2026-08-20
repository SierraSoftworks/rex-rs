use yew::prelude::*;

use super::controls::{Button, Spinner};
use crate::app::{AuthContext, use_auth};
use crate::auth::AuthStatus;

/// Mounts its children only once we know the caller may see them.
///
/// Every view that reads data is wrapped in one of these, so no view has to
/// carry its own "am I signed in yet?" branch and no view fires a request that
/// is guaranteed to come back 401.
#[derive(Properties, PartialEq)]
pub struct ProtectedProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Protected)]
pub fn protected(props: &ProtectedProps) -> Html {
    let auth: AuthContext = use_auth();

    match &auth.status {
        AuthStatus::Loading => html! {
            <div class="fill">
                <div class="center">
                    <Spinner label="Checking your session" />
                </div>
            </div>
        },
        AuthStatus::SignedIn(_) | AuthStatus::Disabled => {
            html! { <>{ for props.children.iter() }</> }
        }
        AuthStatus::NeedsLogin => {
            let on_login = auth.on_login.clone();

            html! {
                <div class="fill">
                    <div class="center">
                        <p>
                            { "We notice you're not yet authenticated. You'll need to " }
                            <Button
                                kind={crate::components::controls::ButtonKind::Primary}
                                onclick={Callback::from(move |_: MouseEvent| on_login.emit(()))}>
                                { "Login" }
                            </Button>
                            { " before you can continue." }
                        </p>
                    </div>
                </div>
            }
        }
        AuthStatus::Forbidden(message) => html! {
            <div class="fill">
                <div class="center">
                    <h1>{ "No entry" }</h1>
                    <p>{ message }</p>
                </div>
            </div>
        },
        AuthStatus::Error(message) => html! {
            <div class="fill">
                <div class="center">
                    <h1>{ "Something went wrong" }</h1>
                    <p>{ message }</p>
                </div>
            </div>
        },
    }
}
