use yew::prelude::*;

use crate::{auth, components::controls::Spinner};

/// The page the sign-in popup lands on.
///
/// It exchanges its authorization code through the server, leaves the result
/// where the window that opened it can find it, and closes itself. Nothing else
/// in the application mounts here.
#[function_component(Callback)]
pub fn callback() -> Html {
    use_effect_with((), |_| {
        wasm_bindgen_futures::spawn_local(auth::complete_login());
    });

    html! {
        <div class="fill">
            <div class="center">
                <Spinner label="Signing you in" />
            </div>
        </div>
    }
}
