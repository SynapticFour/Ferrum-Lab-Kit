use std::sync::{Arc, RwLock};
use std::time::Duration;

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
    discovery: Arc<RwLock<Option<OidcDiscoveryDocument>>>,
    jwks: Arc<RwLock<Option<JwkSetDocument>>>,
    http: reqwest::blocking::Client,
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

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("reqwest TLS backend")
}

fn map_id_token_algorithm(alg: Algorithm) -> Result<Algorithm, AuthError> {
    match alg {
        Algorithm::RS256
        | Algorithm::RS384
        | Algorithm::RS512
        | Algorithm::ES256
        | Algorithm::ES384 => Ok(alg),
        other => Err(AuthError::Oidc(format!(
            "unsupported ID token algorithm {other:?}"
        ))),
    }
}

impl LsLoginOidc {
    pub fn new(cfg: LsLoginConfig) -> Self {
        let mut cfg = cfg;
        cfg.client_secret = lab_kit_core::resolve_oidc_client_secret(
            &cfg.client_secret,
            lab_kit_core::LS_LOGIN_CLIENT_SECRET_ENV,
        );
        Self {
            cfg,
            discovery: Arc::new(RwLock::new(None)),
            jwks: Arc::new(RwLock::new(None)),
            http: http_client(),
        }
    }

    fn discovery_url(&self) -> Result<Url, AuthError> {
        let mut base = self.cfg.issuer.trim_end_matches('/').to_string();
        if !base.ends_with("/.well-known/openid-configuration") {
            base.push_str("/.well-known/openid-configuration");
        }
        Url::parse(&base).map_err(|e| AuthError::Oidc(format!("issuer URL: {e}")))
    }

    fn read_cache<T: Clone>(lock: &RwLock<Option<T>>) -> Result<Option<T>, AuthError> {
        lock.read()
            .map(|g| g.clone())
            .map_err(|_| AuthError::Oidc("OIDC cache lock poisoned".into()))
    }

    fn write_cache<T>(lock: &RwLock<Option<T>>, value: T) -> Result<(), AuthError> {
        let mut w = lock
            .write()
            .map_err(|_| AuthError::Oidc("OIDC cache lock poisoned".into()))?;
        *w = Some(value);
        Ok(())
    }

    /// Fetch `.well-known/openid-configuration` (cached in memory).
    pub fn fetch_discovery(&self) -> Result<OidcDiscoveryDocument, AuthError> {
        if let Some(d) = Self::read_cache(&self.discovery)? {
            return Ok(d);
        }
        let url = self.discovery_url()?;
        let doc: OidcDiscoveryDocument = self
            .http
            .get(url.as_str())
            .send()?
            .error_for_status()?
            .json()?;
        Self::write_cache(&self.discovery, doc.clone())?;
        Ok(doc)
    }

    fn load_jwks(&self, jwks_uri: &str) -> Result<JwkSetDocument, AuthError> {
        if let Some(j) = Self::read_cache(&self.jwks)? {
            return Ok(j);
        }
        let doc: JwkSetDocument = self.http.get(jwks_uri).send()?.error_for_status()?.json()?;
        Self::write_cache(&self.jwks, doc.clone())?;
        Ok(doc)
    }

    /// Validate ID token signature against JWKS, issuer, audience (`client_id`).
    pub fn validate_id_token_blocking(&self, token: &str) -> Result<Value, AuthError> {
        let disc = self.fetch_discovery()?;
        let jwks = self.load_jwks(&disc.jwks_uri)?;
        let header = decode_header(token)?;
        let alg = map_id_token_algorithm(header.alg)?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::Oidc("JWT missing kid".into()))?;
        let key_json = jwks
            .keys
            .iter()
            .find(|k| k.get("kid").and_then(|v| v.as_str()) == Some(kid.as_str()))
            .ok_or_else(|| AuthError::Oidc(format!("no JWK for kid={kid}")))?;
        let jwk: Jwk = serde_json::from_value(key_json.clone())?;
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
        let client = LsLoginOidc {
            cfg: self.cfg.clone(),
            discovery: Arc::clone(&self.discovery),
            jwks: Arc::clone(&self.jwks),
            http: self.http.clone(),
        };
        let token = token.to_string();
        tokio::task::spawn_blocking(move || client.validate_id_token_blocking(&token))
            .await
            .map_err(|e| AuthError::Oidc(e.to_string()))?
    }
}

/// Keycloak is OIDC-compatible; reuse LS Login discovery + JWKS validation.
pub struct KeycloakOidc {
    inner: LsLoginOidc,
}

impl KeycloakOidc {
    pub fn new(cfg: lab_kit_core::KeycloakConfig) -> Result<Self, AuthError> {
        let secret = lab_kit_core::resolve_oidc_client_secret(
            cfg.client_secret.as_deref().unwrap_or(""),
            lab_kit_core::KEYCLOAK_CLIENT_SECRET_ENV,
        );
        if cfg.client_id.trim().is_empty() || secret.trim().is_empty() {
            return Err(AuthError::Config(
                "keycloak requires client_id and client_secret (set KEYCLOAK_CLIENT_SECRET)".into(),
            ));
        }
        let ls = lab_kit_core::LsLoginConfig {
            client_id: cfg.client_id,
            client_secret: secret,
            issuer: cfg.issuer.to_string(),
            redirect_uri: None,
            scopes: LsLoginOidc::default_scopes()
                .into_iter()
                .map(String::from)
                .collect(),
        };
        Ok(Self {
            inner: LsLoginOidc::new(ls),
        })
    }
}

#[async_trait]
impl AuthProvider for KeycloakOidc {
    fn name(&self) -> &'static str {
        "keycloak"
    }

    async fn validate_id_token(&self, token: &str) -> Result<Value, AuthError> {
        self.inner.validate_id_token(token).await
    }
}

/// LDAP bind is not implemented; callers must use LS Login or Keycloak.
pub struct LdapAuth;

#[async_trait]
impl AuthProvider for LdapAuth {
    fn name(&self) -> &'static str {
        "ldap"
    }

    async fn validate_id_token(&self, _token: &str) -> Result<Value, AuthError> {
        Err(AuthError::Config(
            "LDAP bind authentication is not implemented; use ls-login or keycloak".into(),
        ))
    }
}
