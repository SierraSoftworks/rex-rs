use actix_web::{FromRequest, HttpRequest, dev::Payload};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use openidconnect::{ClientId, IdToken, IdTokenClaims, Nonce, NonceVerifier, RedirectUrl, core::{CoreClient, CoreGenderClaim, CoreJsonWebKeyType, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreProviderMetadata}, reqwest::http_client};
use std::sync::Arc;
use super::APIError;
use futures::{FutureExt, future::{ready, Ready}};

lazy_static! {
    static ref CLIENT: Arc<CoreClient> = Arc::new(get_client());
}

pub type AuthIdToken = IdToken<AuthAdditionalClaims, CoreGenderClaim, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreJsonWebKeyType>;

pub type AuthIdTokenClaims = IdTokenClaims<AuthAdditionalClaims, CoreGenderClaim>;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    claims: AuthIdTokenClaims,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AuthAdditionalClaims {
    pub oid: String,
    pub roles: Vec<String>,
    pub scp: String,
    pub unique_name: String,
}

impl openidconnect::AdditionalClaims for AuthAdditionalClaims {

}

impl AuthToken {
    pub fn oid(&self) -> &str {
        &self.claims.additional_claims().oid
    }

    pub fn name(&self) -> String {
        self.claims.name().and_then(|n| n.get(None)).map(|n| n.to_string()).unwrap_or(String::new())
    }

    pub fn roles(&self) -> &Vec<String> {
        &self.claims.additional_claims().roles
    }

    pub fn scopes(&self) -> Vec<&str> {
        self.claims.additional_claims().scp.split(" ").collect()
    }

    pub fn email(&self) -> &str {
        &self.claims.additional_claims().unique_name
    }

    fn from_request_internal(req: &HttpRequest, payload: &mut Payload) -> Result<AuthToken, APIError> {
        let get_creds = BearerAuth::from_request(req, payload).now_or_never();
        let creds = get_creds
            .ok_or(APIError::unauthorized())?
            .map_err(|_| APIError::unauthorized())?;
            
        let client = CLIENT.clone();
        
        let id_token: AuthIdToken = serde_json::from_value(serde_json::json!(creds.token())).map_err(|e| {
            warn!("Unable to deserialize credential token: {}", e);
            APIError::unauthorized()
        })?;

        #[cfg(not(test))]
        let token_verifier = client.id_token_verifier();
        
        #[cfg(test)]
        let token_verifier = client.id_token_verifier().insecure_disable_signature_check().require_issuer_match(false).require_audience_match(false);

        let nonce_verifier = NoOpNonceVerifier{};

        let claims: &AuthIdTokenClaims = id_token.claims(&token_verifier, nonce_verifier)
        .map_err(|e| {
            warn!("Unable to verify ID token for incoming request: {}", e);
            APIError::unauthorized()
        })?;

        Ok(AuthToken {
            claims: claims.clone()
        })
    }
}

impl FromRequest for AuthToken {
    type Error = APIError;
    type Future = Ready<Result<AuthToken, APIError>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        ready(AuthToken::from_request_internal(req, payload))
    }
}

fn get_client() -> CoreClient {
    let issuer_url = openidconnect::IssuerUrl::new("https://sts.windows.net/a26571f1-22b3-4756-ac7b-39ca684fab48/".to_string()).expect("The issuer URL should parse correctly.");
    let provider_metadata = CoreProviderMetadata::discover(
        &issuer_url,
        http_client
    )
    .expect("We should be able to resolve provider metadata for Azure AD.");

    let redirect_url = RedirectUrl::new("https://rex.sierrasoftworks.com".to_string()).expect("The redirect URL should parse correctly");

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new("https://rex.sierrasoftworks.com".to_string()),
        None)
        .set_redirect_uri(redirect_url);

    client
}

struct NoOpNonceVerifier {}

impl NonceVerifier for NoOpNonceVerifier {
    fn verify(self, _nonce: Option<&Nonce>) -> Result<(), String> {
        Ok(())
    }
}