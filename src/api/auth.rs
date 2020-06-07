use actix_web::{Error, dev::{Extensions, ServiceRequest, Payload, ServiceResponse}, HttpMessage, HttpRequest, FromRequest};
use actix_web_httpauth::extractors::bearer;
use biscuit::{CompactJson};
use oidc::token::Jws;
use oidc::{issuer, Client};
use std::{sync::Arc, pin::Pin, task::{Poll, Context}};
use super::APIError;
use futures::{Future, future::{ready, Ready}};
use actix_service::{Transform, Service};

lazy_static! {
    static ref CLIENT: Arc<Client> = Arc::new(get_client());
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
    fn set_token(token: AuthToken, req: &ServiceRequest) {
        let mut exts = req.extensions_mut();

        debug!("Adding AuthToken to the request context: aud={} oid={}", token.aud, token.oid);
        exts.insert(token);
    }

    fn get_token(extensions: &mut Extensions) -> Option<Self> {
        let token_box: Option<&AuthToken> = extensions.get();

        if token_box.is_none() {
            warn!("Attempted to fetch AuthToken for a request which did not have an associated auth token.");
        }

        token_box.map(|t| t.clone())
    }
}

impl FromRequest for AuthToken {
    type Error = APIError;
    type Future = Ready<Result<AuthToken, APIError>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(AuthToken::get_token(&mut *req.extensions_mut()).ok_or(APIError::unauthorized()))
    }
}

impl CompactJson for AuthToken {}

pub struct Auth;

impl<S, B> Transform<S> for Auth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthTokenMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthTokenMiddleware { service }))
    }
}

pub struct AuthTokenMiddleware<S> {
    service: S,
}

impl <S,B> Service for AuthTokenMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S:: Future: 'static,
    B: 'static
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        req.headers().get("authorization")
            .ok_or_else(||{
                debug!("No authorization header present on request.");
                APIError::unauthorized()
            })
            .and_then(|t| t.to_str().map_err(|err| {
                debug!("Unable to convert authorization header into a string: {}", err);
                APIError::unauthorized()
            }))
            .and_then(|t| t.splitn(2, " ").nth(1).ok_or_else(|| {
                debug!("Authorization header did not contain the required Bearer <token> components");
                APIError::unauthorized()
            }))
            .map(|p| Jws::new_encoded(p))
            .and_then(|token: Jws<AuthToken, biscuit::Empty>| token.unverified_payload().map_err(|err| {
                warn!("Authorization token could not be parsed correctly: {}", err);
                APIError::unauthorized()
            }))
            .map(|t| AuthToken::set_token(t, &req))
            .unwrap_or(());

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

pub async fn auth_validator(
    req: ServiceRequest,
    creds: bearer::BearerAuth,
) -> Result<ServiceRequest, Error> {
    let mut token = Jws::new_encoded(creds.token());

    let client = CLIENT.clone();

    client
        .decode_token(&mut token)
        .and_then(|()| client.validate_token(&token, None, None))
        .map(|_| req)
        .map_err(|_e| 
            APIError::new(401, "Unauthorized", "You have not provided a valid authentication token. Please authenticate and try again.").into())
}

fn get_client() -> Client {
    return Client::discover(
        "https://rex.sierrasoftworks.com".to_string(),
        "".to_string(),
        reqwest::Url::parse("https://rex.sierrasoftworks.com").expect("a valid redirect URL"),
        issuer::microsoft_tenant("a26571f1-22b3-4756-ac7b-39ca684fab48"),
    )
    .expect("an AzureAD client");
}
