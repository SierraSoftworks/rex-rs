use serde::{Deserialize, Serialize};

/// Everything the browser needs to start an authorization code flow, served by
/// `GET /api/v1/auth/metadata`.
///
/// The server reads this from its cached OIDC discovery document so that the
/// browser never has to fetch the provider's metadata cross-origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthMetadata {
    /// `None` when the deployment runs with authentication disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    /// The redirect URI the popup must land on, so the client and server agree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    pub enabled: bool,
}

/// `POST /api/v1/auth/token` — exchanges an authorization code for tokens.
///
/// The exchange is performed server-side because it needs the client secret;
/// the browser only ever sees the resulting tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenRequest {
    pub code: String,
    /// Present when the client used PKCE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_verifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}

/// `POST /api/v1/auth/refresh` — trades a refresh token for a fresh ID token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// The tokens handed back to the browser. The **ID token** is the bearer Rex
/// accepts on its API; the access token is not used by Rex itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenResponse {
    pub id_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Seconds until `id_token` expires, when the provider tells us.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
}

/// `GET /api/v1/auth/me` — who the server thinks you are, after validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPrincipal {
    /// The 32-character hex principal id used everywhere else in the API.
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(rename = "emailHash")]
    pub email_hash: String,
}
