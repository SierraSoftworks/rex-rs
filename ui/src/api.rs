//! The typed client for Rex's API.
//!
//! Every request goes through [`request`], which is the only place the bearer
//! token is attached. That is deliberate: a page that forgot the header would
//! not fail loudly, it would quietly behave as though you were signed out.

use gloo_net::http::{Method, RequestBuilder};
use rex_api::{ApiError, CollectionV3, IdeaV3, RoleAssignmentV3, UserV3};
use serde::{Serialize, de::DeserializeOwned};

use crate::auth;

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    request(Method::GET, path, None::<&()>, true).await
}

pub async fn post<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, ApiError> {
    request(Method::POST, path, Some(body), true).await
}

pub async fn put<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, ApiError> {
    request(Method::PUT, path, Some(body), true).await
}

pub async fn delete(path: &str) -> Result<(), ApiError> {
    request::<(), Empty>(Method::DELETE, path, None, true)
        .await
        .map(|_| ())
}

/// The auth endpoints themselves, which must work before there is a token.
pub async fn get_unauthenticated<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    request(Method::GET, path, None::<&()>, false).await
}

pub async fn post_unauthenticated<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    request(Method::POST, path, Some(body), false).await
}

async fn request<B: Serialize, T: DeserializeOwned>(
    method: Method,
    path: &str,
    body: Option<&B>,
    authenticated: bool,
) -> Result<T, ApiError> {
    let response = send(method.clone(), path, body, authenticated).await?;

    // One retry, and only after a refresh actually succeeded: a 401 that
    // survives a fresh token is a real 401.
    if authenticated && response.0 == 401 && auth::refresh().await.is_ok() {
        let retried = send(method, path, body, authenticated).await?;
        return finish(retried);
    }

    finish(response)
}

struct Response(u16, String);

async fn send<B: Serialize>(
    method: Method,
    path: &str,
    body: Option<&B>,
    authenticated: bool,
) -> Result<Response, ApiError> {
    let mut builder = RequestBuilder::new(path).method(method);

    if authenticated && let Some(token) = auth::token() {
        builder = builder.header("Authorization", &format!("Bearer {token}"));
    }

    let request = match body {
        Some(body) => builder.json(body).map_err(|err| {
            ApiError::new(
                400,
                "Bad Request",
                &format!("We could not encode the request: {err}"),
            )
        })?,
        None => builder.build().map_err(transport_error)?,
    };

    let response = request.send().await.map_err(transport_error)?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    Ok(Response(status, text))
}

fn finish<T: DeserializeOwned>(response: Response) -> Result<T, ApiError> {
    let Response(status, body) = response;

    if !(200..300).contains(&status) {
        // The server sends `{code, error, message}`; anything else came from a
        // proxy or a crash, so fall back to something a person can act on.
        return Err(serde_json::from_str::<ApiError>(&body).unwrap_or_else(|_| {
            ApiError::new(
                status,
                "Request Failed",
                if body.is_empty() {
                    "The server did not explain what went wrong."
                } else {
                    body.as_str()
                },
            )
        }));
    }

    // 204 No Content is the normal answer to a delete.
    if body.trim().is_empty() {
        return serde_json::from_str("null").map_err(|_| {
            ApiError::new(
                500,
                "Unexpected Response",
                "The server returned nothing where a result was expected.",
            )
        });
    }

    serde_json::from_str(&body).map_err(|err| {
        ApiError::new(
            500,
            "Unexpected Response",
            &format!("We could not read the server's response: {err}"),
        )
    })
}

fn transport_error(err: gloo_net::Error) -> ApiError {
    ApiError::new(
        503,
        "Service Unavailable",
        &format!("We could not reach the server: {err}"),
    )
}

/// The Gravatar-compatible hash of an email address.
fn email_hash(email: &str) -> String {
    format!("{:x}", md5::compute(email.to_lowercase().trim().as_bytes()))
}

/// The unit response of a request that returns no body.
#[derive(serde::Deserialize)]
#[serde(transparent)]
struct Empty(#[allow(dead_code)] Option<()>);

// ── Resource helpers ─────────────────────────────────────────────────────────

pub async fn collections() -> Result<Vec<CollectionV3>, ApiError> {
    get("/api/v3/collections").await
}

pub async fn collection(id: &str) -> Result<CollectionV3, ApiError> {
    get(&format!("/api/v3/collection/{id}")).await
}

pub async fn new_collection(name: &str) -> Result<CollectionV3, ApiError> {
    post(
        "/api/v3/collections",
        &CollectionV3 {
            id: None,
            user_id: None,
            name: name.to_string(),
        },
    )
    .await
}

pub async fn ideas(collection: &str) -> Result<Vec<IdeaV3>, ApiError> {
    get(&format!("/api/v3/collection/{collection}/ideas")).await
}

pub async fn idea(collection: &str, id: &str) -> Result<IdeaV3, ApiError> {
    get(&format!("/api/v3/collection/{collection}/idea/{id}")).await
}

/// A random idea from `collection`, or from the caller's default collection
/// when none is given.
pub async fn random_idea(collection: Option<&str>) -> Result<IdeaV3, ApiError> {
    match collection {
        Some(collection) => get(&format!("/api/v3/collection/{collection}/idea/random")).await,
        None => get("/api/v3/idea/random").await,
    }
}

pub async fn new_idea(collection: Option<&str>, idea: &IdeaV3) -> Result<IdeaV3, ApiError> {
    match collection {
        Some(collection) => post(&format!("/api/v3/collection/{collection}/ideas"), idea).await,
        None => post("/api/v3/ideas", idea).await,
    }
}

pub async fn store_idea(idea: &IdeaV3) -> Result<IdeaV3, ApiError> {
    let collection = idea.collection.clone().unwrap_or_default();
    let id = idea.id.clone().unwrap_or_default();

    put(&format!("/api/v3/collection/{collection}/idea/{id}"), idea).await
}

pub async fn remove_idea(collection: &str, id: &str) -> Result<(), ApiError> {
    delete(&format!("/api/v3/collection/{collection}/idea/{id}")).await
}

pub async fn role_assignments(collection: &str) -> Result<Vec<RoleAssignmentV3>, ApiError> {
    get(&format!("/api/v3/collection/{collection}/users")).await
}

pub async fn set_role_assignment(
    collection: &str,
    user: &str,
    role: &str,
) -> Result<RoleAssignmentV3, ApiError> {
    put(
        &format!("/api/v3/collection/{collection}/user/{user}"),
        &RoleAssignmentV3 {
            collection_id: None,
            user_id: None,
            role: role.to_string(),
        },
    )
    .await
}

pub async fn remove_role_assignment(collection: &str, user: &str) -> Result<(), ApiError> {
    delete(&format!("/api/v3/collection/{collection}/user/{user}")).await
}

/// Looks a person up by the MD5 of their email address.
///
/// The hash is computed here so the address itself never leaves the browser --
/// and it is a lookup key, not a secret: MD5 over an email address is
/// reversible by anyone with a word list.
pub async fn user_by_email(email: &str) -> Result<UserV3, ApiError> {
    get(&format!("/api/v3/user/{}", email_hash(email))).await
}
