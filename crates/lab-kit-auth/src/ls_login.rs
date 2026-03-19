use std::sync::RwLock;

use async_trait::async_trait;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use lab_kit_core::LsLoginConfig;
use serde::Deserialize;
use serde_json::Value;
use url::Url;

use crate::provider::AuthProvider;
use crate::AuthError;

/// ELIXIR LS Login (Life Science AAI) — OIDC discovery, JWKS, JWT validation.
pub struct LsLoginOidc {
    cfg: LsLoginConfig,
    discovery: RwLock<Option<OidcDiscoveryDocument>>,
    jwks: RwLock<Option<JwkSetDocument>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OidcDiscoveryDocument {
    pub issuer: String,
    pub jwks_uri: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwkSetDocument {
    pub keys: Vec<Value>,
}

impl LsLoginOidc {
    pub fn new(cfg: LsLoginConfig) -> Self {
        Self {
            cfg,
            discovery: RwLock::new(None),
            jwks: RwLock::new(None),
        }
    }

    fn discovery_url(&self) -> Result<Url, AuthError> {
        let mut base = self.cfg.issuer.trim_end_matches('/').to_string();
        if !base.ends_with("/.well-known/openid-configuration") {
            base.push_str("/.well-known/openid-configuration");
        }
        Url::parse(&base).map_err(|e| AuthError::Oidc(format!("issuer URL: {e}")))
    }

    /// Fetch `.well-known/openid-configuration` (cached in memory).
    pub fn fetch_discovery(&self) -> Result<OidcDiscoveryDocument, AuthError> {
        if let Some(d) = self.discovery.read().ok().and_then(|g| g.clone()) {
            return Ok(d);
        }
        let url = self.discovery_url()?;
        let doc: OidcDiscoveryDocument = reqwest::blocking::get(url.as_str())?
            .error_for_status()?
            .json()?;
        if let Ok(mut w) = self.discovery.write() {
            *w = Some(doc.clone());
        }
        Ok(doc)
    }

    fn load_jwks(&self, jwks_uri: &str) -> Result<JwkSetDocument, AuthError> {
        if let Some(j) = self.jwks.read().ok().and_then(|g| g.clone()) {
            return Ok(j);
        }
        let doc: JwkSetDocument = reqwest::blocking::get(jwks_uri)?
            .error_for_status()?
            .json()?;
        if let Ok(mut w) = self.jwks.write() {
            *w = Some(doc.clone());
        }
        Ok(doc)
    }

    /// Validate ID token signature against JWKS, issuer, audience (`client_id`).
    pub fn validate_id_token_blocking(&self, token: &str) -> Result<Value, AuthError> {
        let disc = self.fetch_discovery()?;
        let jwks = self.load_jwks(&disc.jwks_uri)?;
        let header = decode_header(token)?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::Oidc("JWT missing kid".into()))?;
        let key_json = jwks
            .keys
            .iter()
            .find(|k| k.get("kid").and_then(|v| v.as_str()) == Some(kid.as_str()))
            .ok_or_else(|| AuthError::Oidc(format!("no JWK for kid={kid}")))?;
        let jwk: Jwk = serde_json::from_value(key_json.clone())?;
        let alg = match header.alg {
            jsonwebtoken::Algorithm::RS256 => Algorithm::RS256,
            jsonwebtoken::Algorithm::RS384 => Algorithm::RS384,
            jsonwebtoken::Algorithm::RS512 => Algorithm::RS512,
            jsonwebtoken::Algorithm::ES256 => Algorithm::ES256,
            jsonwebtoken::Algorithm::ES384 => Algorithm::ES384,
            _ => Algorithm::RS256,
        };
        let decoding_key =
            DecodingKey::from_jwk(&jwk).map_err(|e| AuthError::Oidc(e.to_string()))?;
        let mut validation = Validation::new(alg);
        validation.set_audience(&[&self.cfg.client_id]);
        validation.set_issuer(&[&disc.issuer]);
        let data = decode::<Value>(token, &decoding_key, &validation)?;
        Ok(data.claims)
    }

    /// Recommended scopes for Authorization Code + PKCE with refresh tokens.
    pub fn default_scopes() -> Vec<&'static str> {
        vec![
            "openid",
            "profile",
            "email",
            "offline_access",
            "ga4gh_passport_v1",
        ]
    }
}

#[async_trait]
impl AuthProvider for LsLoginOidc {
    fn name(&self) -> &'static str {
        "ls-login"
    }

    async fn validate_id_token(&self, token: &str) -> Result<Value, AuthError> {
        let cfg = self.cfg.clone();
        let token = token.to_string();
        tokio::task::spawn_blocking(move || {
            let client = LsLoginOidc::new(cfg);
            client.validate_id_token_blocking(&token)
        })
        .await
        .map_err(|e| AuthError::Oidc(e.to_string()))?
    }
}
