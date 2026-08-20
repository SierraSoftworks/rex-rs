use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use arc_swap::ArcSwapOption;
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    IdToken, IdTokenClaims, IssuerUrl, Nonce, NonceVerifier, OAuth2TokenResponse, PkceCodeVerifier,
    RedirectUrl, RefreshToken, Scope,
    core::{
        CoreClient, CoreGenderClaim, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
        CoreProviderMetadata,
    },
};
use rex_api::{ApiError, AuthMetadata};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument, warn};

use super::{Acl, AuthToken};
use crate::config::{Config, OidcConfig};

/// Every claim the provider sent that isn't part of the OIDC standard set.
///
/// Rex needs these because both the principal id claim and the access control
/// expressions are configurable: which claims matter is a deployment's choice,
/// not something the code can know up front.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExtraClaims(pub HashMap<String, serde_json::Value>);

impl openidconnect::AdditionalClaims for ExtraClaims {}

pub type RexIdToken = IdToken<
    ExtraClaims,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;

pub type RexIdTokenClaims = IdTokenClaims<ExtraClaims, CoreGenderClaim>;

type RexOidcClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

/// The tokens handed back to the browser after a code exchange or refresh.
pub struct TokenSet {
    pub id_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
}

struct CachedProvider {
    metadata: CoreProviderMetadata,
    fetched_at: Instant,
}

/// Cached OIDC discovery plus everything derived from it.
///
/// Discovery (and the JWKS it names) is fetched once and reused until its TTL
/// expires. A token signed with a key we have never seen forces an early
/// refetch, so provider key rotation resolves itself within one request instead
/// of within one TTL.
pub struct OidcState {
    config: Option<OidcConfig>,
    base_url: Option<String>,
    user_acl: Option<Acl>,
    admin_acl: Option<Acl>,
    http: openidconnect::reqwest::Client,
    cached: ArcSwapOption<CachedProvider>,
    /// Serialises discovery refreshes so a burst of requests triggers one fetch.
    refreshing: tokio::sync::Mutex<()>,
}

impl OidcState {
    pub fn new(config: &Config) -> Result<Self, String> {
        let http = openidconnect::reqwest::ClientBuilder::new()
            // Following redirects on a token endpoint is how credentials end up
            // somewhere they were never meant to go.
            .redirect(openidconnect::reqwest::redirect::Policy::none())
            .build()
            .map_err(|err| format!("We could not build an HTTP client for OIDC: {err}"))?;

        let user_acl = config
            .web
            .auth
            .user_acl
            .as_deref()
            .map(Acl::new)
            .transpose()
            .map_err(|err| format!("The `web.auth.user_acl` expression is not valid: {err}"))?;

        let admin_acl = config
            .web
            .auth
            .admin_acl
            .as_deref()
            .map(Acl::new)
            .transpose()
            .map_err(|err| format!("The `web.auth.admin_acl` expression is not valid: {err}"))?;

        if config.web.auth.oidc.is_none() {
            warn!(
                "No `[web.auth.oidc]` section is configured, so Rex is running with authentication disabled. Every request will be treated as the local developer."
            );
        }

        Ok(Self {
            config: config.web.auth.oidc.clone(),
            base_url: config.web.base_url.clone(),
            user_acl,
            admin_acl,
            http,
            cached: ArcSwapOption::empty(),
            refreshing: tokio::sync::Mutex::new(()),
        })
    }

    pub fn enabled(&self) -> bool {
        self.config.is_some()
    }

    pub fn config(&self) -> Option<&OidcConfig> {
        self.config.as_ref()
    }

    pub fn user_acl(&self) -> Option<&Acl> {
        self.user_acl.as_ref()
    }

    pub fn admin_acl(&self) -> Option<&Acl> {
        self.admin_acl.as_ref()
    }

    /// Where the identity provider sends the popup once the user has consented.
    pub fn redirect_uri(&self) -> String {
        let base = self
            .base_url
            .as_deref()
            .unwrap_or("http://localhost:8000")
            .trim_end_matches('/');

        format!("{base}/auth/callback")
    }

    fn require_config(&self) -> Result<&OidcConfig, ApiError> {
        self.config.as_ref().ok_or_else(|| {
            ApiError::new(
                501,
                "Not Implemented",
                "This Rex instance is running with authentication disabled.",
            )
        })
    }

    async fn provider(&self) -> Result<Arc<CachedProvider>, ApiError> {
        let config = self.require_config()?;
        let ttl = Duration::from_secs(config.discovery_ttl_seconds);

        if let Some(cached) = self.cached.load_full()
            && cached.fetched_at.elapsed() < ttl
        {
            return Ok(cached);
        }

        self.refresh_provider().await
    }

    #[instrument(name = "oidc.discover", skip(self), err)]
    async fn refresh_provider(&self) -> Result<Arc<CachedProvider>, ApiError> {
        let config = self.require_config()?;

        // One fetch per burst: whoever loses the race takes the winner's result.
        let _guard = self.refreshing.lock().await;

        if let Some(cached) = self.cached.load_full()
            && cached.fetched_at.elapsed() < Duration::from_secs(5)
        {
            return Ok(cached);
        }

        let issuer = IssuerUrl::new(config.endpoint.clone()).map_err(|err| {
            error!({ exception.message = %err }, "The configured OIDC endpoint is not a valid URL.");
            ApiError::internal_server_error()
        })?;

        let metadata = CoreProviderMetadata::discover_async(issuer, &self.http)
            .await
            .map_err(|err| {
                error!({ exception.message = %err }, "We could not fetch the OIDC provider's discovery document.");
                ApiError::new(
                    503,
                    "Service Unavailable",
                    "We could not reach the identity provider. Please try again shortly.",
                )
            })?;

        info!("Refreshed the OIDC provider's discovery document.");

        let cached = Arc::new(CachedProvider {
            metadata,
            fetched_at: Instant::now(),
        });
        self.cached.store(Some(cached.clone()));

        Ok(cached)
    }

    fn client(&self, provider: &CachedProvider) -> Result<RexOidcClient, ApiError> {
        let config = self.require_config()?;

        let client = CoreClient::from_provider_metadata(
            provider.metadata.clone(),
            ClientId::new(config.client_id.clone()),
            config.client_secret.clone().map(ClientSecret::new),
        )
        .set_redirect_uri(RedirectUrl::new(self.redirect_uri()).map_err(|err| {
            error!({ exception.message = %err }, "The configured base URL does not produce a valid redirect URI.");
            ApiError::internal_server_error()
        })?);

        Ok(client)
    }

    /// What the browser needs in order to start a login.
    pub async fn metadata(&self) -> Result<AuthMetadata, ApiError> {
        let Some(config) = self.config.as_ref() else {
            return Ok(AuthMetadata {
                authorization_endpoint: None,
                client_id: None,
                scopes: Vec::new(),
                redirect_uri: None,
                enabled: false,
            });
        };

        let provider = self.provider().await?;

        Ok(AuthMetadata {
            authorization_endpoint: Some(provider.metadata.authorization_endpoint().to_string()),
            client_id: Some(config.client_id.clone()),
            scopes: config.scopes.clone(),
            redirect_uri: Some(self.redirect_uri()),
            enabled: true,
        })
    }

    /// Validates a bearer ID token, refetching discovery once if the token was
    /// signed with a key we do not recognise.
    #[instrument(name = "oidc.validate", skip_all, err)]
    pub async fn validate(&self, raw: &str) -> Result<AuthToken, ApiError> {
        let config = self.require_config()?;
        let provider = self.provider().await?;

        match self.verify_with(&provider, raw, config) {
            Ok(token) => Ok(token),
            Err(err) => {
                // Only worth a second attempt if the cached keys could actually
                // be stale.
                if provider.fetched_at.elapsed() < Duration::from_secs(60) {
                    return Err(err);
                }

                let provider = self.refresh_provider().await?;
                self.verify_with(&provider, raw, config)
            }
        }
    }

    fn verify_with(
        &self,
        provider: &CachedProvider,
        raw: &str,
        config: &OidcConfig,
    ) -> Result<AuthToken, ApiError> {
        let client = self.client(provider)?;

        let id_token: RexIdToken =
            serde_json::from_value(serde_json::Value::String(raw.to_string())).map_err(|err| {
                warn!(
                    "The bearer token could not be parsed as an ID token: {}",
                    err
                );
                ApiError::unauthorized()
            })?;

        // Restricting the accepted algorithms to asymmetric signatures shuts the
        // door on algorithm confusion, where a token signed with HMAC over a
        // known public key is presented as though it came from the provider.
        let verifier = client.id_token_verifier().set_allowed_algs([
            CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
            CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha384,
            CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha512,
            CoreJwsSigningAlgorithm::RsaSsaPssSha256,
            CoreJwsSigningAlgorithm::RsaSsaPssSha384,
            CoreJwsSigningAlgorithm::RsaSsaPssSha512,
            CoreJwsSigningAlgorithm::EcdsaP256Sha256,
            CoreJwsSigningAlgorithm::EcdsaP384Sha384,
            CoreJwsSigningAlgorithm::EcdsaP521Sha512,
        ]);

        let claims = id_token
            .claims(&verifier, NoOpNonceVerifier)
            .map_err(|err| {
                warn!(
                    "We could not verify the ID token on an incoming request: {}",
                    err
                );
                ApiError::unauthorized()
            })?;

        AuthToken::from_claims(claims.clone(), config)
    }

    /// The confidential half of the authorization code flow.
    #[instrument(name = "oidc.exchange_code", skip_all, err)]
    pub async fn exchange_code(
        &self,
        code: String,
        verifier: Option<String>,
    ) -> Result<TokenSet, ApiError> {
        let provider = self.provider().await?;
        let client = self.client(&provider)?;

        let mut request = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|err| configuration_error("exchange an authorization code", err))?;

        if let Some(verifier) = verifier {
            request = request.set_pkce_verifier(PkceCodeVerifier::new(verifier));
        }

        let response = request.request_async(&self.http).await.map_err(|err| {
            warn!("The authorization code exchange was rejected: {}", err);
            ApiError::unauthorized()
        })?;

        token_set(&response)
    }

    #[instrument(name = "oidc.refresh_token", skip_all, err)]
    pub async fn refresh(&self, refresh_token: String) -> Result<TokenSet, ApiError> {
        let provider = self.provider().await?;
        let client = self.client(&provider)?;

        let response = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .map_err(|err| configuration_error("refresh a token", err))?
            .add_scopes(
                self.require_config()?
                    .scopes
                    .iter()
                    .map(|scope| Scope::new(scope.clone())),
            )
            .request_async(&self.http)
            .await
            .map_err(|err| {
                warn!("The refresh token exchange was rejected: {}", err);
                ApiError::unauthorized()
            })?;

        token_set(&response)
    }
}

fn configuration_error(what: &str, err: impl std::fmt::Display) -> ApiError {
    error!({ exception.message = %err }, "We could not {} because the provider metadata is incomplete.", what);
    ApiError::internal_server_error()
}

fn token_set(response: &openidconnect::core::CoreTokenResponse) -> Result<TokenSet, ApiError> {
    let id_token = response
        .extra_fields()
        .id_token()
        .ok_or_else(|| {
            warn!("The identity provider returned a token response without an ID token.");
            ApiError::unauthorized()
        })?
        .to_string();

    Ok(TokenSet {
        id_token,
        refresh_token: response.refresh_token().map(|t| t.secret().clone()),
        expires_in: response.expires_in().map(|d| d.as_secs()),
    })
}

/// Rex never issues a nonce, because it never starts the flow — the browser
/// does, and it verifies its own `state` before the code ever reaches us.
struct NoOpNonceVerifier;

impl NonceVerifier for NoOpNonceVerifier {
    fn verify(self, _nonce: Option<&Nonce>) -> Result<(), String> {
        Ok(())
    }
}
