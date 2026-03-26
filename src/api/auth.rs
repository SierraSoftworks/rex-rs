use super::APIError;
use actix::prelude::*;
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use openidconnect::{
    core::{
        CoreClient, CoreGenderClaim, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
        CoreProviderMetadata,
    },
    ClientId, EndpointMaybeSet, EndpointNotSet, EndpointSet, IdToken, IdTokenClaims,
    Nonce, NonceVerifier, RedirectUrl,
};
use std::{future::Future, pin::Pin};

/// The OIDC client type with endpoints configured by Azure AD's discovery document.
/// The authorization endpoint (HasAuthUrl) is always set by the provider metadata,
/// while the token URL and userinfo URL may or may not be present (EndpointMaybeSet).
type OidcClient = CoreClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointMaybeSet, EndpointMaybeSet>;

pub type AuthIdToken = IdToken<
    AuthAdditionalClaims,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;

pub type AuthIdTokenClaims = IdTokenClaims<AuthAdditionalClaims, CoreGenderClaim>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    claims: AuthIdTokenClaims,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuthAdditionalClaims {
    pub oid: String,
    pub roles: Vec<String>,
    pub scp: String,
    pub unique_name: String,
}

impl openidconnect::AdditionalClaims for AuthAdditionalClaims {}

impl AuthToken {
    pub fn oid(&self) -> &str {
        &self.claims.additional_claims().oid
    }

    pub fn name(&self) -> String {
        self.claims
            .name()
            .and_then(|n| n.get(None))
            .map(|n| n.to_string())
            .unwrap_or_default()
    }

    pub fn roles(&self) -> &Vec<String> {
        &self.claims.additional_claims().roles
    }

    pub fn scopes(&self) -> Vec<&str> {
        self.claims.additional_claims().scp.split(' ').collect()
    }

    pub fn email(&self) -> &str {
        &self.claims.additional_claims().unique_name
    }

    fn bearer_token_from_request(req: &HttpRequest) -> Result<String, APIError> {
        req.headers()
            .get("Authorization")
            .ok_or_else(APIError::unauthorized)
            .and_then(|header| header.to_str().map_err(|_| APIError::unauthorized()))
            .and_then(|header| {
                if header.starts_with("Bearer ") {
                    header
                        .split_ascii_whitespace()
                        .nth(1)
                        .ok_or_else(APIError::unauthorized)
                } else {
                    Err(APIError::unauthorized())
                }
            })
            .map(|s| s.to_string())
    }

    #[instrument("auth_token.from_request", skip(req))]
    async fn from_request_internal(req: HttpRequest) -> Result<AuthToken, APIError> {
        let token = AuthToken::bearer_token_from_request(&req)?;

        let actor = req
            .app_data::<web::Data<Addr<OidcActor>>>()
            .ok_or_else(|| {
                error!("OidcActor address not registered in app data");
                APIError::new(
                    500,
                    "Internal Server Error",
                    "We ran into a problem, this has been reported and will be looked at.",
                )
            })?
            .clone();

        actor.send(VerifyToken(token)).await?
    }
}

impl FromRequest for AuthToken {
    type Error = APIError;
    type Future = Pin<Box<dyn Future<Output = Result<AuthToken, APIError>>>>;

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(AuthToken::from_request_internal(req))
    }
}

// ── Actor ─────────────────────────────────────────────────────────────────────

pub struct OidcActor {
    client: Option<OidcClient>,
}

impl OidcActor {
    pub fn new() -> Self {
        Self { client: None }
    }
}

impl Actor for OidcActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.wait(
            actix::fut::wrap_future(get_client()).map(|client, actor: &mut OidcActor, _ctx| {
                actor.client = Some(client);
            }),
        );
    }
}

// ── Message ───────────────────────────────────────────────────────────────────

pub struct VerifyToken(pub String);

impl Message for VerifyToken {
    type Result = Result<AuthToken, APIError>;
}

impl Handler<VerifyToken> for OidcActor {
    type Result = Result<AuthToken, APIError>;

    fn handle(&mut self, msg: VerifyToken, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.as_ref().ok_or_else(|| {
            error!("OidcActor received VerifyToken before client was initialized");
            APIError::new(
                503,
                "Service Unavailable",
                "The authentication service is not yet ready. Please try again shortly.",
            )
        })?;

        let id_token: AuthIdToken =
            serde_json::from_value(serde_json::json!(msg.0.as_str())).map_err(|e| {
                warn!("Unable to deserialize credential token: {}", e);
                APIError::unauthorized()
            })?;

        #[cfg(not(test))]
        let token_verifier = client.id_token_verifier();

        #[cfg(test)]
        let token_verifier = client
            .id_token_verifier()
            .insecure_disable_signature_check()
            .require_issuer_match(false)
            .require_audience_match(false);

        let nonce_verifier = NoOpNonceVerifier {};

        let claims: &AuthIdTokenClaims =
            id_token
                .claims(&token_verifier, nonce_verifier)
                .map_err(|e| {
                    warn!("Unable to verify ID token for incoming request: {}", e);
                    APIError::unauthorized()
                })?;

        Ok(AuthToken {
            claims: claims.clone(),
        })
    }
}

// ── OIDC discovery ────────────────────────────────────────────────────────────

async fn get_client() -> OidcClient {
    let issuer_url = openidconnect::IssuerUrl::new(
        "https://sts.windows.net/a26571f1-22b3-4756-ac7b-39ca684fab48/".to_string(),
    )
    .expect("The issuer URL should parse correctly.");
    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build HTTP client for OpenID Connect discovery");
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
        .await
        .expect("We should be able to resolve provider metadata for Azure AD.");

    let redirect_url = RedirectUrl::new("https://rex.sierrasoftworks.com".to_string())
        .expect("The redirect URL should parse correctly");

    CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new("https://rex.sierrasoftworks.com".to_string()),
        None,
    )
    .set_redirect_uri(redirect_url)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

struct NoOpNonceVerifier {}

impl NonceVerifier for NoOpNonceVerifier {
    fn verify(self, _nonce: Option<&Nonce>) -> Result<(), String> {
        Ok(())
    }
}
