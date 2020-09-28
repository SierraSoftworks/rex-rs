use actix_web::{FromRequest, HttpRequest, dev::Payload};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use biscuit::{CompactJson};
use oidc::token::Jws;
use oidc::{issuer, Client};
use std::sync::Arc;
use super::APIError;
use futures::{FutureExt, future::{ready, Ready}};

lazy_static! {
    static ref CLIENT: Arc<Client> = Arc::new(get_client());
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct AuthToken {
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub sub: String,
    pub name: String,
    pub oid: String,
    pub roles: Vec<String>,
    pub scp: String,
    pub unique_name: String,
}

impl AuthToken {
    fn from_request_internal(req: &HttpRequest, payload: &mut Payload) -> Result<AuthToken, APIError> {
        let get_creds = BearerAuth::from_request(req, payload).now_or_never();
        let creds = get_creds
            .ok_or(APIError::unauthorized())?
            .map_err(|_| APIError::unauthorized())?;
        
        #[cfg(not(test))]
        {
            let mut ticket = Jws::new_encoded(creds.token());
            let client = CLIENT.clone();
            client
                .decode_token(&mut ticket)
                .and_then(|()| client.validate_token(&ticket, None, None))
                .map_err(|_e| APIError::unauthorized())?;
        }
        
        let ticket: Jws<AuthToken, biscuit::Empty> = Jws::new_encoded(creds.token());
        let token = ticket.unverified_payload().map_err(|_| APIError::unauthorized())?;
        
        Ok(token)
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

impl CompactJson for AuthToken {}

fn get_client() -> Client {
    return Client::discover(
        "https://rex.sierrasoftworks.com".to_string(),
        "".to_string(),
        reqwest::Url::parse("https://rex.sierrasoftworks.com").expect("a valid redirect URL"),
        issuer::microsoft_tenant("a26571f1-22b3-4756-ac7b-39ca684fab48"),
    )
    .expect("an AzureAD client");
}
