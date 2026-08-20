use yew::prelude::*;
use yew_router::prelude::*;

use super::{
    Icon,
    controls::{Avatar, Tooltip},
    gravatar,
    icons::IconView,
};
use crate::{
    api,
    app::{AuthContext, Route, use_auth},
};

/// The hairline-bordered bar across the top: wordmark, collection breadcrumb,
/// and the menu.
#[function_component(Header)]
pub fn header() -> Html {
    let auth: AuthContext = use_auth();
    let route = use_route::<Route>();

    let collection = route.as_ref().and_then(collection_of);
    let principal = auth.status.principal();

    // The breadcrumb names the collection, which means asking the server for it.
    // A failed lookup simply leaves the breadcrumb off rather than raising
    // anything: the header is chrome, and the view below is already reporting
    // whatever went wrong.
    let name = use_state(|| None::<String>);
    {
        let name = name.clone();
        let collection = collection.clone();
        let ready = auth.status.is_ready();
        // The path is in the dependencies as well as the collection id: several
        // routes share one collection, and a caller's default collection may not
        // exist until a view they have just visited brings it into being.
        let path = use_location()
            .map(|l| l.path().to_string())
            .unwrap_or_default();

        use_effect_with((collection, ready, path), move |(collection, ready, _)| {
            let Some(cid) = collection.clone().filter(|_| *ready) else {
                name.set(None);
                return;
            };

            wasm_bindgen_futures::spawn_local(async move {
                name.set(api::collection(&cid).await.ok().map(|c| c.name));
            });
        });
    }

    html! {
        <header class="header">
            <h2 class="header__title">
                <Link<Route> to={Route::Home} classes="header__name">{ "REX" }</Link<Route>>

                if let Some(name) = (*name).clone() {
                    <span class="header__separator">{ "/" }</span>
                    <span class="header__collection">{ name }</span>
                }
            </h2>

            <nav class="header-menu">
                if let Some(cid) = collection.clone() {
                    <Tooltip content="New Idea">
                        <Link<Route> to={Route::NewIdea { cid: cid.clone() }}>
                            <IconView icon={Icon::Plus} />
                        </Link<Route>>
                    </Tooltip>
                }

                if auth.status.is_ready() {
                    <Tooltip content="New Collection">
                        <Link<Route> to={Route::NewCollection}>
                            <IconView icon={Icon::DocumentAdd} />
                        </Link<Route>>
                    </Tooltip>

                    <Tooltip content="View Collections">
                        <Link<Route> to={Route::Collections}>
                            <IconView icon={Icon::DocumentCopy} />
                        </Link<Route>>
                    </Tooltip>
                }

                if let Some(principal) = principal {
                    <div class="header-menu__user">
                        <Avatar
                            size={16}
                            src={gravatar(&principal.email_hash, 50)}
                            alt={principal.name.clone()} />
                        { first_name(&principal.name) }
                    </div>

                    // The previous interface had a logout action defined but no
                    // way to reach it.
                    <Tooltip content="Sign Out">
                        <button
                            type="button"
                            class="header-menu__button"
                            aria-label="Sign Out"
                            onclick={{
                                let on_logout = auth.on_logout.clone();
                                Callback::from(move |_: MouseEvent| on_logout.emit(()))
                            }}>
                            <IconView icon={Icon::SignOut} />
                        </button>
                    </Tooltip>
                }
            </nav>
        </header>
    }
}

/// The collection the current route is scoped to, if any.
fn collection_of(route: &Route) -> Option<String> {
    match route {
        Route::Collection { cid }
        | Route::Idea { cid, .. }
        | Route::NewIdea { cid }
        | Route::Manage { cid }
        | Route::Invite { cid } => Some(cid.clone()),
        _ => None,
    }
}

fn first_name(name: &str) -> String {
    name.split(' ').next().unwrap_or(name).to_string()
}
