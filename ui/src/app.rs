//! The application shell: routes, the auth context, and the page frame.

use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    auth::{self, AuthStatus},
    components::{Header, NotificationProvider},
    views,
};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/collection/:cid")]
    Collection { cid: String },
    #[at("/collection/:cid/idea/:iid")]
    Idea { cid: String, iid: String },
    #[at("/collection/:cid/new")]
    NewIdea { cid: String },
    #[at("/collection/:cid/manage")]
    Manage { cid: String },
    #[at("/collection/:cid/invite")]
    Invite { cid: String },
    #[at("/collections")]
    Collections,
    #[at("/collections/new")]
    NewCollection,
    #[at("/auth/callback")]
    AuthCallback,

    // The gallery exists only in debug builds -- route, switch arm, module, and
    // re-export are all gated, so a release binary contains none of it.
    #[cfg(debug_assertions)]
    #[at("/demo/controls")]
    Gallery,
    #[cfg(debug_assertions)]
    #[at("/demo/controls/:control")]
    GalleryControl { control: String },

    #[not_found]
    #[at("/404")]
    NotFound,
}

/// What the auth context carries: the current status, and the two things any
/// component might want to do about it.
#[derive(Clone, PartialEq)]
pub struct AuthContext {
    pub status: AuthStatus,
    pub on_login: Callback<()>,
    pub on_logout: Callback<()>,
}

#[hook]
pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("an AuthProvider above this component")
}

#[function_component(App)]
pub fn app() -> Html {
    // The sign-in popup lands on the callback route. It has no shell, no auth
    // context, and nothing to fetch -- it exchanges its code, hands the tokens
    // back to the window that opened it, and closes.
    if auth::window()
        .location()
        .pathname()
        .is_ok_and(|path| path == "/auth/callback")
    {
        return html! { <views::Callback /> };
    }

    html! {
        <BrowserRouter>
            <NotificationProvider>
                <AuthProvider>
                    <Shell />
                </AuthProvider>
            </NotificationProvider>
        </BrowserRouter>
    }
}

#[derive(Properties, PartialEq)]
struct AuthProviderProps {
    children: Children,
}

#[function_component(AuthProvider)]
fn auth_provider(props: &AuthProviderProps) -> Html {
    let status = use_state(|| AuthStatus::Loading);

    // Resolve the session once, on mount. Everything downstream waits on this
    // rather than each view deciding for itself whether it is signed in.
    {
        let status = status.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                status.set(match auth::probe().await {
                    Ok(resolved) => resolved,
                    Err(message) => AuthStatus::Error(message),
                });
            });
        });
    }

    let on_login = {
        let status = status.clone();

        Callback::from(move |_: ()| {
            let status = status.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Err(message) = auth::begin_login().await {
                    status.set(AuthStatus::Error(message));
                    return;
                }

                status.set(match auth::probe().await {
                    Ok(resolved) => resolved,
                    Err(message) => AuthStatus::Error(message),
                });
            });
        })
    };

    let on_logout = {
        let status = status.clone();

        Callback::from(move |_: ()| {
            auth::forget_tokens();
            status.set(AuthStatus::NeedsLogin);
        })
    };

    let context = AuthContext {
        status: (*status).clone(),
        on_login,
        on_logout,
    };

    html! {
        <ContextProvider<AuthContext> context={context}>
            { for props.children.iter() }
        </ContextProvider<AuthContext>>
    }
}

#[function_component(Shell)]
fn shell() -> Html {
    let location = use_location();

    html! {
        <div class="app">
            <Header />
            <main class="content">
                // Keyed by path so that navigating between two instances of the
                // same view remounts it, which is what the old interface did and
                // what the views' load-on-mount effects assume.
                <Switch<Route>
                    key={location.map(|l| l.path().to_string()).unwrap_or_default()}
                    render={switch} />
            </main>
        </div>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <views::Home collection={None::<String>} /> },
        Route::Collection { cid } => html! { <views::Home collection={Some(cid)} /> },
        Route::Idea { cid, iid } => html! { <views::Idea collection={cid} idea={iid} /> },
        Route::NewIdea { cid } => html! { <views::NewIdea collection={cid} /> },
        Route::Manage { cid } => html! { <views::Manage collection={cid} /> },
        Route::Invite { cid } => html! { <views::Invite collection={cid} /> },
        Route::Collections => html! { <views::Collections /> },
        Route::NewCollection => html! { <views::NewCollection /> },
        Route::AuthCallback => html! { <views::Callback /> },

        #[cfg(debug_assertions)]
        Route::Gallery => html! { <views::Gallery control={None::<String>} /> },
        #[cfg(debug_assertions)]
        Route::GalleryControl { control } => html! { <views::Gallery control={Some(control)} /> },

        Route::NotFound => html! {
            <div class="fill">
                <div class="center">
                    <h1>{ "Nothing here" }</h1>
                    <p>{ "The page you were looking for does not exist." }</p>
                </div>
            </div>
        },
    }
}
