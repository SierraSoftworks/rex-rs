//! A real OIDC provider, served by wiremock, for the authentication tests.
//!
//! It publishes a genuine discovery document and JWKS, and mints genuine RS256
//! ID tokens — which means the tests exercise the same verification path that
//! ships. There is deliberately no way to weaken the verifier from a test: the
//! previous implementation carried a `#[cfg(test)]`
//! `insecure_disable_signature_check`, and code that only *usually* checks
//! signatures is one build flag away from not checking them at all.

use std::collections::HashMap;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::{RsaPrivateKey, pkcs8::DecodePrivateKey, traits::PublicKeyParts};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

/// A fixed key, so tests are deterministic and nothing spends a second
/// generating RSA parameters. It signs nothing outside this test suite.
const SIGNING_KEY: &str = include_str!("assets/test-signing-key.pem");

const KEY_ID: &str = "rex-test-key";

pub struct TestIdentityProvider {
    server: MockServer,
    key: RsaPrivateKey,
    client_id: String,
}

impl TestIdentityProvider {
    /// Starts a provider whose tokens are addressed to `client_id`.
    pub async fn start(client_id: &str) -> Self {
        let server = MockServer::start().await;
        let key = RsaPrivateKey::from_pkcs8_pem(SIGNING_KEY).expect("the test key should parse");

        let issuer = server.uri();

        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "issuer": issuer,
                "authorization_endpoint": format!("{issuer}/authorize"),
                "token_endpoint": format!("{issuer}/token"),
                "jwks_uri": format!("{issuer}/jwks"),
                "response_types_supported": ["code"],
                "subject_types_supported": ["public"],
                "id_token_signing_alg_values_supported": ["RS256"],
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/jwks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "keys": [jwk(&key)],
            })))
            .mount(&server)
            .await;

        Self {
            server,
            key,
            client_id: client_id.to_string(),
        }
    }

    /// The issuer URL to put in `[web.auth.oidc] endpoint`.
    pub fn endpoint(&self) -> String {
        self.server.uri()
    }

    /// Mints a valid ID token carrying the supplied claims on top of the
    /// standard ones.
    pub fn id_token(&self, subject: &str, extra: HashMap<&str, serde_json::Value>) -> String {
        self.token_with(subject, extra, Algorithm::RS256, 300)
    }

    /// Mints a token that expired five minutes ago.
    pub fn expired_id_token(&self, subject: &str) -> String {
        self.token_with(subject, HashMap::new(), Algorithm::RS256, -300)
    }

    fn token_with(
        &self,
        subject: &str,
        extra: HashMap<&str, serde_json::Value>,
        algorithm: Algorithm,
        expires_in: i64,
    ) -> String {
        let now = chrono::Utc::now().timestamp();

        let mut claims = serde_json::Map::new();
        claims.insert("iss".into(), json!(self.server.uri()));
        claims.insert("aud".into(), json!(self.client_id));
        claims.insert("sub".into(), json!(subject));
        claims.insert("iat".into(), json!(now));
        claims.insert("exp".into(), json!(now + expires_in));

        for (name, value) in extra {
            claims.insert(name.into(), value);
        }

        let mut header = Header::new(algorithm);
        header.kid = Some(KEY_ID.into());

        let key = EncodingKey::from_rsa_pem(SIGNING_KEY.as_bytes())
            .expect("the test key should load for signing");

        jsonwebtoken::encode(&header, &claims, &key).expect("the test token should sign")
    }

    /// Mints a token signed with HMAC over the provider's public modulus — the
    /// classic algorithm-confusion attack, which verification must refuse.
    pub fn hmac_forged_token(&self, subject: &str) -> String {
        let now = chrono::Utc::now().timestamp();

        let claims = json!({
            "iss": self.server.uri(),
            "aud": self.client_id,
            "sub": subject,
            "iat": now,
            "exp": now + 300,
        });

        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some(KEY_ID.into());

        let secret = self.key.n().to_bytes_be();

        jsonwebtoken::encode(&header, &claims, &EncodingKey::from_secret(&secret))
            .expect("the forged token should sign")
    }
}

fn jwk(key: &RsaPrivateKey) -> serde_json::Value {
    let public = key.to_public_key();

    json!({
        "kty": "RSA",
        "use": "sig",
        "alg": "RS256",
        "kid": KEY_ID,
        "n": URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
        "e": URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
    })
}
