//! Signing in, staying signed in, and knowing who you are.
//!
//! Rex runs the authorization code flow in a popup and lets the server perform
//! the confidential exchange, so the browser never holds a client secret. What
//! it does hold is the **ID token**, in `sessionStorage`, which it sends as a
//! bearer. Nothing is stored in a cookie, so there is no CSRF surface.

use std::cell::RefCell;

use futures::{
    FutureExt,
    future::{LocalBoxFuture, Shared},
};
use rex_api::{AuthMetadata, AuthPrincipal, RefreshRequest, TokenRequest, TokenResponse};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

/// The bearer we send with every request.
const TOKEN_KEY: &str = "rex.token";
/// The refresh token, used to get a new bearer without another popup.
const REFRESH_KEY: &str = "rex.refresh";
/// The CSRF-style nonce tying an authorization response to the request we made.
const STATE_KEY: &str = "rex.oidc.state";
/// Where the popup leaves its result for the window that opened it.
///
/// This lives in `localStorage` rather than `sessionStorage` because the two
/// windows do not share the latter reliably.
const POPUP_RESULT_KEY: &str = "rex.oidc.popup_result";

pub fn window() -> web_sys::Window {
    web_sys::window().expect("a browser window")
}

fn session_storage() -> Option<web_sys::Storage> {
    window().session_storage().ok().flatten()
}

fn local_storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

fn read(storage: Option<web_sys::Storage>, key: &str) -> Option<String> {
    storage?.get_item(key).ok().flatten()
}

fn write(storage: Option<web_sys::Storage>, key: &str, value: &str) {
    if let Some(storage) = storage {
        let _ = storage.set_item(key, value);
    }
}

fn remove(storage: Option<web_sys::Storage>, key: &str) {
    if let Some(storage) = storage {
        let _ = storage.remove_item(key);
    }
}

pub fn token() -> Option<String> {
    read(session_storage(), TOKEN_KEY)
}

fn refresh_token() -> Option<String> {
    read(session_storage(), REFRESH_KEY)
}

fn store_tokens(tokens: &TokenResponse) {
    write(session_storage(), TOKEN_KEY, &tokens.id_token);

    // A refresh grant that returns no new refresh token leaves the old one in
    // place, rather than logging the user out at the next expiry.
    if let Some(refresh) = tokens.refresh_token.as_deref() {
        write(session_storage(), REFRESH_KEY, refresh);
    }
}

pub fn forget_tokens() {
    remove(session_storage(), TOKEN_KEY);
    remove(session_storage(), REFRESH_KEY);
}

/// Where the whole application is served from, used to build the redirect URI.
fn origin() -> String {
    window()
        .location()
        .origin()
        .unwrap_or_else(|_| String::from("http://localhost:8000"))
}

fn random_state() -> String {
    let mut bytes = [0u8; 24];

    if let Ok(crypto) = window().crypto() {
        let _ = crypto.get_random_values_with_u8_array(&mut bytes);
    }

    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ── Single-flight refresh ────────────────────────────────────────────────────

/// A refresh that several callers can await at once.
type SharedRefresh = Shared<LocalBoxFuture<'static, Result<(), String>>>;

thread_local! {
    /// The in-flight refresh, if there is one.
    ///
    /// Several requests failing with a 401 at once is the normal case -- a view
    /// fires three fetches on mount -- and each of them independently asking
    /// for a new token would burn the refresh token and log the user out. They
    /// all wait on this instead.
    static IN_FLIGHT: RefCell<Option<SharedRefresh>> = const { RefCell::new(None) };
}

/// Trades the refresh token for a new bearer, collapsing concurrent callers
/// into a single request.
pub async fn refresh() -> Result<(), String> {
    let shared = IN_FLIGHT.with(|cell| {
        if let Some(existing) = cell.borrow().as_ref() {
            return existing.clone();
        }

        let future = perform_refresh().boxed_local().shared();
        *cell.borrow_mut() = Some(future.clone());
        future
    });

    let result = shared.await;

    IN_FLIGHT.with(|cell| *cell.borrow_mut() = None);

    result
}

async fn perform_refresh() -> Result<(), String> {
    let Some(refresh_token) = refresh_token() else {
        return Err("There is no refresh token to renew your session with.".into());
    };

    let tokens: TokenResponse =
        crate::api::post_unauthenticated("/api/v1/auth/refresh", &RefreshRequest { refresh_token })
            .await
            .map_err(|err| err.message)?;

    store_tokens(&tokens);

    Ok(())
}

// ── Sign-in ──────────────────────────────────────────────────────────────────

/// Opens the identity provider in a popup and waits for it to hand back tokens.
///
/// Only ever call this from an explicit click. A popup opened in response to a
/// background request is exactly what pop-up blockers exist to stop, and the
/// old interface did precisely that whenever a signed-out session made an API
/// call.
pub async fn begin_login() -> Result<(), String> {
    let metadata: AuthMetadata = crate::api::get_unauthenticated("/api/v1/auth/metadata")
        .await
        .map_err(|err| err.message)?;

    if !metadata.enabled {
        return Ok(());
    }

    let (Some(endpoint), Some(client_id)) = (metadata.authorization_endpoint, metadata.client_id)
    else {
        return Err("This Rex instance did not describe its identity provider.".into());
    };

    let state = random_state();
    write(local_storage(), STATE_KEY, &state);
    remove(local_storage(), POPUP_RESULT_KEY);

    let redirect_uri = metadata
        .redirect_uri
        .unwrap_or_else(|| format!("{}/auth/callback", origin()));

    let url = format!(
        "{endpoint}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        encode(&client_id),
        encode(&redirect_uri),
        encode(&metadata.scopes.join(" ")),
        encode(&state),
    );

    window()
        .open_with_url_and_target_and_features(&url, "rex-login", "width=520,height=680")
        .map_err(|_| String::from("We could not open the sign-in window."))?
        .ok_or_else(|| {
            String::from(
                "The sign-in window was blocked. Please allow pop-ups for this site and try again.",
            )
        })?;

    await_popup_result().await
}

/// Polls the handoff slot until the popup writes to it, or until it is clear
/// nothing is coming.
async fn await_popup_result() -> Result<(), String> {
    // Roughly five minutes, which is longer than any provider's sign-in page
    // stays useful.
    for _ in 0..600 {
        gloo_timers::future::TimeoutFuture::new(500).await;

        let Some(raw) = read(local_storage(), POPUP_RESULT_KEY) else {
            continue;
        };

        remove(local_storage(), POPUP_RESULT_KEY);
        remove(local_storage(), STATE_KEY);

        let outcome: PopupResult = serde_json::from_str(&raw).map_err(|_| {
            String::from("The sign-in window returned something we could not read.")
        })?;

        return match outcome {
            PopupResult::Success(tokens) => {
                store_tokens(&tokens);
                Ok(())
            }
            PopupResult::Failure { message } => Err(message),
        };
    }

    Err("The sign-in window did not complete in time.".into())
}

/// Runs in the popup once the provider redirects back to `/auth/callback`.
pub async fn complete_login() {
    let outcome = exchange_callback().await;

    let payload = match outcome {
        Ok(tokens) => PopupResult::Success(tokens),
        Err(message) => PopupResult::Failure { message },
    };

    if let Ok(raw) = serde_json::to_string(&payload) {
        write(local_storage(), POPUP_RESULT_KEY, &raw);
    }

    let _ = window().close();
}

async fn exchange_callback() -> Result<TokenResponse, String> {
    let query = window()
        .location()
        .search()
        .map_err(|_| String::from("We could not read the sign-in response."))?;

    let params = parse_query(&query);

    if let Some(error) = params.get("error") {
        let description = params
            .get("error_description")
            .cloned()
            .unwrap_or_else(|| error.clone());
        return Err(description);
    }

    let expected = read(local_storage(), STATE_KEY);
    let received = params.get("state").cloned();

    // The state check is what ties this response to the request this browser
    // actually made; without it, anyone can feed us an authorization code.
    if expected.is_none() || expected != received {
        return Err("The sign-in response did not match the request we made.".into());
    }

    let code = params
        .get("code")
        .cloned()
        .ok_or_else(|| String::from("The sign-in response carried no authorization code."))?;

    crate::api::post_unauthenticated(
        "/api/v1/auth/token",
        &TokenRequest {
            code,
            code_verifier: None,
            redirect_uri: None,
        },
    )
    .await
    .map_err(|err| err.message)
}

/// Asks the server who we are, which is also how we discover that a token has
/// stopped being good.
pub async fn probe() -> Result<AuthStatus, String> {
    let metadata: AuthMetadata = crate::api::get_unauthenticated("/api/v1/auth/metadata")
        .await
        .map_err(|err| err.message)?;

    if !metadata.enabled {
        return Ok(AuthStatus::Disabled);
    }

    if token().is_none() {
        return Ok(AuthStatus::NeedsLogin);
    }

    match crate::api::get::<AuthPrincipal>("/api/v1/auth/me").await {
        Ok(principal) => Ok(AuthStatus::SignedIn(principal)),
        Err(err) if err.code == 401 => {
            forget_tokens();
            Ok(AuthStatus::NeedsLogin)
        }
        Err(err) if err.code == 403 => Ok(AuthStatus::Forbidden(err.message)),
        Err(err) => Err(err.message),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthStatus {
    Loading,
    /// The deployment runs without an identity provider.
    Disabled,
    SignedIn(AuthPrincipal),
    NeedsLogin,
    /// A valid token, from somebody this instance will not serve.
    Forbidden(String),
    Error(String),
}

impl AuthStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, AuthStatus::SignedIn(_) | AuthStatus::Disabled)
    }

    pub fn principal(&self) -> Option<&AuthPrincipal> {
        match self {
            AuthStatus::SignedIn(principal) => Some(principal),
            _ => None,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "outcome")]
enum PopupResult {
    Success(TokenResponse),
    Failure { message: String },
}

fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    query
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty())
        .filter_map(|pair| pair.split_once('='))
        .map(|(name, value)| (name.to_string(), decode(value)))
        .collect()
}

/// Percent-encodes a query parameter value.
///
/// `encodeURIComponent` is right here and correct, which beats hand-rolling the
/// character classes.
fn encode(value: &str) -> String {
    js_sys::encode_uri_component(value).into()
}

fn decode(value: &str) -> String {
    let value = value.replace('+', " ");
    js_sys::decode_uri_component(&value)
        .map(String::from)
        .unwrap_or(value)
}

/// Reads the current value out of an input event's target.
///
/// Not auth's business, strictly, but every form in the app needs it and this
/// is the only module that already reaches for `web_sys`.
pub fn input_value(event: &web_sys::Event) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}
