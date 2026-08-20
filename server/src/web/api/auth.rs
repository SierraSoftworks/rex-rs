use std::sync::Arc;

use actix_web::web;
use rex_api::{ApiError, AuthMetadata, AuthPrincipal, RefreshRequest, TokenRequest, TokenResponse};
use tracing::instrument;

use crate::{
    auth::{AuthToken, OidcState, TokenSet},
    services::Services,
    web::ApiResponse,
};

pub fn configure<S: Services>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/v1/auth/metadata")
            .name("auth_metadata_v1")
            .route(web::get().to(auth_metadata_v1)),
    )
    .service(
        web::resource("/api/v1/auth/token")
            .name("auth_token_v1")
            .route(web::post().to(auth_token_v1)),
    )
    .service(
        web::resource("/api/v1/auth/refresh")
            .name("auth_refresh_v1")
            .route(web::post().to(auth_refresh_v1)),
    )
    .service(
        web::resource("/api/v1/auth/me")
            .name("auth_me_v1")
            .route(web::get().to(auth_me_v1::<S>)),
    );
}

/// Everything the browser needs to begin a login.
///
/// Serving this from Rex rather than letting the browser read the provider's
/// discovery document keeps the whole flow same-origin until the popup opens.
#[instrument(err, skip_all, fields(otel.kind = "internal"))]
async fn auth_metadata_v1(
    oidc: web::Data<Arc<OidcState>>,
) -> Result<ApiResponse<AuthMetadata>, ApiError> {
    oidc.metadata().await.map(ApiResponse)
}

/// The confidential half of the authorization code flow.
///
/// The browser never holds the client secret; it hands us the code it received
/// and gets tokens back.
#[instrument(err, skip_all, fields(otel.kind = "internal"))]
async fn auth_token_v1(
    request: web::Json<TokenRequest>,
    oidc: web::Data<Arc<OidcState>>,
) -> Result<ApiResponse<TokenResponse>, ApiError> {
    let request = request.into_inner();

    oidc.exchange_code(request.code, request.code_verifier)
        .await
        .map(|tokens| ApiResponse(response(tokens)))
}

#[instrument(err, skip_all, fields(otel.kind = "internal"))]
async fn auth_refresh_v1(
    request: web::Json<RefreshRequest>,
    oidc: web::Data<Arc<OidcState>>,
) -> Result<ApiResponse<TokenResponse>, ApiError> {
    oidc.refresh(request.into_inner().refresh_token)
        .await
        .map(|tokens| ApiResponse(response(tokens)))
}

/// Who the server thinks you are.
///
/// The UI probes this rather than decoding the token itself, so that "am I
/// signed in?" is answered by the same validation the API applies.
#[instrument(err, skip_all, fields(otel.kind = "internal"))]
async fn auth_me_v1<S: Services>(
    _services: web::Data<S>,
    token: AuthToken,
) -> Result<ApiResponse<AuthPrincipal>, ApiError> {
    Ok(ApiResponse(token.principal()))
}

fn response(tokens: TokenSet) -> TokenResponse {
    TokenResponse {
        id_token: tokens.id_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
    }
}
