use std::{future::Future, pin::Pin, sync::Arc};

use actix_web::{FromRequest, HttpRequest, dev::Payload, web};
use rex_api::{ApiError, AuthPrincipal};
use tracing::{error, instrument, warn};

use super::{AclContext, OidcState, oidc::RexIdTokenClaims};
use crate::{
    config::OidcConfig,
    models::{Id, email_hash, format_id, parse_id},
};

/// A validated caller.
///
/// Taking one of these as a handler argument is what makes an endpoint
/// authenticated: extraction fails with a 401 when the bearer is missing or
/// invalid, and a 403 when the deployment's `user_acl` denies the principal.
#[derive(Debug, Clone)]
pub struct AuthToken {
    claims: Option<RexIdTokenClaims>,
    principal_id: Id,
    name: String,
    email: String,
}

impl AuthToken {
    pub(super) fn from_claims(
        claims: RexIdTokenClaims,
        config: &OidcConfig,
    ) -> Result<Self, ApiError> {
        let mut token = Self {
            claims: Some(claims),
            principal_id: 0,
            name: String::new(),
            email: String::new(),
        };

        let raw_principal = token.claim(&config.username_claim).ok_or_else(|| {
            warn!(
                "The ID token does not carry the configured `{}` claim, so we cannot identify the caller.",
                config.username_claim
            );
            ApiError::unauthorized()
        })?;

        // Parsed through the same dashes-stripped hex path the service has
        // always used, so an Azure AD `oid` keeps resolving to the principal id
        // its data is already filed under.
        token.principal_id = parse_id(&raw_principal).ok_or_else(|| {
            warn!(
                "The `{}` claim could not be parsed as an id.",
                config.username_claim
            );
            ApiError::unauthorized()
        })?;

        token.email = token.claim(&config.email_claim).unwrap_or_default();
        token.name = token.claim("name").unwrap_or_default();

        Ok(token)
    }

    /// The identity every request runs as when `[web.auth.oidc]` is absent.
    pub fn local_developer() -> Self {
        Self {
            claims: None,
            principal_id: 0,
            name: "Local Developer".into(),
            email: "developer@localhost".into(),
        }
    }

    pub fn principal_id(&self) -> Id {
        self.principal_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    /// The first word of the caller's name, which is what the UI greets people
    /// with and what the users table stores.
    pub fn first_name(&self) -> String {
        self.name.split(' ').next().unwrap_or("").to_string()
    }

    pub fn email_hash(&self) -> Id {
        email_hash(&self.email)
    }

    pub fn principal(&self) -> AuthPrincipal {
        AuthPrincipal {
            id: format_id(self.principal_id),
            name: self.name.clone(),
            email: self.email.clone(),
            email_hash: format_id(self.email_hash()),
        }
    }

    /// Reads a claim by name, whether it is one of the standard OIDC claims or
    /// something the provider added.
    pub fn claim(&self, name: &str) -> Option<String> {
        let claims = self.claims.as_ref()?;

        match name {
            "sub" => Some(claims.subject().to_string()),
            "iss" => Some(claims.issuer().to_string()),
            "name" => claims
                .name()
                .and_then(|n| n.get(None))
                .map(|n| n.to_string()),
            "email" => claims.email().map(|e| e.to_string()),
            "preferred_username" => claims.preferred_username().map(|u| u.to_string()),
            other => claims.additional_claims().0.get(other).map(claim_to_string),
        }
    }

    fn bearer_token(req: &HttpRequest) -> Result<String, ApiError> {
        req.headers()
            .get("Authorization")
            .ok_or_else(ApiError::unauthorized)
            .and_then(|header| header.to_str().map_err(|_| ApiError::unauthorized()))
            .and_then(|header| {
                header
                    .strip_prefix("Bearer ")
                    .map(str::trim)
                    .filter(|token| !token.is_empty())
                    .ok_or_else(ApiError::unauthorized)
            })
            .map(|token| token.to_string())
    }

    #[instrument("auth_token.from_request", skip(req), err)]
    async fn extract(req: HttpRequest) -> Result<AuthToken, ApiError> {
        let oidc = req
            .app_data::<web::Data<Arc<OidcState>>>()
            .ok_or_else(|| {
                error!("The OIDC state was not registered in the application data.");
                ApiError::internal_server_error()
            })?
            .clone();

        if !oidc.enabled() {
            return Ok(AuthToken::local_developer());
        }

        let raw = AuthToken::bearer_token(&req)?;
        let token = oidc.validate(&raw).await?;

        if let Some(acl) = oidc.user_acl() {
            let client_ip = req
                .connection_info()
                .realip_remote_addr()
                .map(str::to_owned);

            let context = AclContext {
                token: &token,
                client_ip: client_ip.as_deref(),
                method: req.method().as_str(),
                path: req.path(),
            };

            if !acl.allows(&context) {
                warn!("A caller holding a valid token was denied by `web.auth.user_acl`.");
                return Err(ApiError::forbidden(
                    "You are not authorized to use this Rex instance. Please contact your administrator for access.",
                ));
            }
        }

        Ok(token)
    }
}

/// Renders a JSON claim value the way a filter expression expects to see it.
fn claim_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

impl FromRequest for AuthToken {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output = Result<AuthToken, ApiError>>>>;

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Box::pin(AuthToken::extract(req.clone()))
    }
}
