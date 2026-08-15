//! GA4GH Passport claim handling (`ga4gh_passport_v1`) and Beacon v2 access tiers.
//!
//! Nested visa JWTs are **signature-verified** against the visa issuer JWKS before
//! any `type` / `value` claim is trusted. Fail closed on bad signatures.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use lab_kit_core::BeaconAccessLevel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::AuthError;

/// High-level visa types referenced by Beacon controlled access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisaType {
    #[serde(rename = "ControlledAccessGrants")]
    ControlledAccessGrants,
    #[serde(rename = "ResearcherStatus")]
    ResearcherStatus,
    #[serde(rename = "AffiliationAndRole")]
    AffiliationAndRole,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportVisa {
    pub visa_type: VisaType,
    pub asserted_by: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Nested object inside a verified visa JWT (`ga4gh_visa_v1`).
#[derive(Debug, Clone, Deserialize)]
struct Ga4ghVisaV1 {
    #[serde(rename = "type")]
    visa_type: VisaType,
    value: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default, rename = "by")]
    by: Option<String>,
}

/// Supplies a decoding key for a visa JWT `iss` + `kid`.
pub trait VisaKeySource: Send + Sync {
    fn decoding_key(
        &self,
        issuer: &str,
        kid: &str,
        header_alg: Algorithm,
    ) -> Result<DecodingKey, AuthError>;
}

/// Fetches visa-issuer JWKS over HTTP.
///
/// HTTP fetch is **fail-closed** without an issuer allowlist. Cache entries expire
/// after `ttl` (default 10 minutes).
pub struct HttpJwks {
    cache: RwLock<HashMap<String, (Instant, Value)>>,
    http: Option<reqwest::blocking::Client>,
    allowed_issuers: Vec<String>,
    ttl: Duration,
}

impl Default for HttpJwks {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpJwks {
    /// No allowlist — `decoding_key` refuses network I/O.
    pub fn new() -> Self {
        Self::with_allowed_issuers(Vec::new())
    }

    pub fn with_allowed_issuers(issuers: Vec<String>) -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .ok();
        Self {
            cache: RwLock::new(HashMap::new()),
            http,
            allowed_issuers: issuers,
            ttl: Duration::from_secs(600),
        }
    }

    fn issuer_allowed(&self, issuer: &str) -> bool {
        self.allowed_issuers.iter().any(|a| a == issuer)
    }

    fn client(&self) -> Result<&reqwest::blocking::Client, AuthError> {
        self.http
            .as_ref()
            .ok_or_else(|| AuthError::Oidc("HTTP client unavailable".into()))
    }

    fn jwks_url_for_issuer(&self, issuer: &str) -> Result<String, AuthError> {
        let mut base = issuer.trim_end_matches('/').to_string();
        if !base.ends_with("/.well-known/openid-configuration") {
            base.push_str("/.well-known/openid-configuration");
        }
        let doc: Value = self
            .client()?
            .get(&base)
            .send()
            .map_err(AuthError::Http)?
            .error_for_status()
            .map_err(AuthError::Http)?
            .json()
            .map_err(AuthError::Http)?;
        doc.get("jwks_uri")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| AuthError::Oidc(format!("no jwks_uri for issuer {issuer}")))
    }
}

impl VisaKeySource for HttpJwks {
    fn decoding_key(
        &self,
        issuer: &str,
        kid: &str,
        _header_alg: Algorithm,
    ) -> Result<DecodingKey, AuthError> {
        if self.allowed_issuers.is_empty() {
            return Err(AuthError::Oidc(
                "visa JWKS fetch requires an issuer allowlist (HttpJwks::with_allowed_issuers)"
                    .into(),
            ));
        }
        if !self.issuer_allowed(issuer) {
            return Err(AuthError::Oidc(format!(
                "visa issuer {issuer} is not allowlisted"
            )));
        }

        if let Ok(g) = self.cache.read() {
            if let Some((at, doc)) = g.get(issuer) {
                if at.elapsed() < self.ttl {
                    return key_from_jwks_doc(doc, kid);
                }
            }
        } else {
            return Err(AuthError::Oidc("JWKS cache lock poisoned".into()));
        }

        let jwks_uri = self.jwks_url_for_issuer(issuer)?;
        let doc: Value = self
            .client()?
            .get(&jwks_uri)
            .send()?
            .error_for_status()?
            .json()?;
        let key = key_from_jwks_doc(&doc, kid)?;
        match self.cache.write() {
            Ok(mut w) => {
                w.insert(issuer.to_string(), (Instant::now(), doc));
            }
            Err(_) => return Err(AuthError::Oidc("JWKS cache lock poisoned".into())),
        }
        Ok(key)
    }
}

fn key_from_jwks_doc(doc: &Value, kid: &str) -> Result<DecodingKey, AuthError> {
    let keys = doc
        .get("keys")
        .and_then(|k| k.as_array())
        .ok_or_else(|| AuthError::Oidc("JWKS missing keys".into()))?;
    let key_json = keys
        .iter()
        .find(|k| k.get("kid").and_then(|v| v.as_str()) == Some(kid))
        .ok_or_else(|| AuthError::Oidc(format!("no JWK for kid={kid}")))?;
    let jwk: Jwk = serde_json::from_value(key_json.clone())?;
    DecodingKey::from_jwk(&jwk).map_err(|e| AuthError::Oidc(e.to_string()))
}

fn allowed_visa_algorithm(alg: Algorithm) -> Result<Algorithm, AuthError> {
    match alg {
        Algorithm::RS256
        | Algorithm::RS384
        | Algorithm::RS512
        | Algorithm::ES256
        | Algorithm::ES384 => Ok(alg),
        #[cfg(test)]
        Algorithm::HS256 => Ok(alg),
        other => Err(AuthError::Oidc(format!(
            "unsupported visa JWT algorithm {other:?}"
        ))),
    }
}

/// Decode a visa JWT only after verifying signature against `keys`.
pub fn verify_visa_jwt(token: &str, keys: &dyn VisaKeySource) -> Result<PassportVisa, AuthError> {
    let header = decode_header(token)?;
    let alg = allowed_visa_algorithm(header.alg)?;
    let kid = header
        .kid
        .ok_or_else(|| AuthError::Oidc("visa JWT missing kid".into()))?;

    // Unverified payload is used solely to learn `iss` so we can fetch JWKS.
    // Claims are not trusted until `decode` succeeds below.
    let iss = unverified_issuer(token)?;
    let decoding_key = keys.decoding_key(&iss, &kid, alg)?;
    let mut validation = Validation::new(alg);
    validation.set_issuer(&[&iss]);
    validation.validate_aud = false;
    let data = decode::<Value>(token, &decoding_key, &validation)?;
    visa_from_verified_payload(&data.claims)
        .ok_or_else(|| AuthError::Oidc("visa JWT missing ga4gh_visa_v1".into()))
}

fn unverified_issuer(token: &str) -> Result<String, AuthError> {
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(|| AuthError::Oidc("malformed JWT".into()))?;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| AuthError::Oidc("visa JWT payload is not base64".into()))?;
    let v: Value = serde_json::from_slice(&bytes)?;
    v.get("iss")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| AuthError::Oidc("visa JWT missing iss".into()))
}

pub fn visa_from_verified_payload(claims: &Value) -> Option<PassportVisa> {
    let inner = claims.get("ga4gh_visa_v1")?;
    let visa: Ga4ghVisaV1 = serde_json::from_value(inner.clone()).ok()?;
    Some(PassportVisa {
        visa_type: visa.visa_type,
        asserted_by: visa.by.or_else(|| visa.source.clone()).unwrap_or_default(),
        value: visa.value,
        source: visa.source,
    })
}

pub struct VisaEvaluator;

impl VisaEvaluator {
    /// Verify each nested visa JWT in `ga4gh_passport_v1`. Invalid entries are skipped (fail closed).
    pub fn visas_from_claims(claims: &Value) -> Vec<PassportVisa> {
        Self::visas_from_claims_with(claims, &HttpJwks::new())
    }

    pub fn visas_from_claims_with(claims: &Value, keys: &dyn VisaKeySource) -> Vec<PassportVisa> {
        let Some(passport) = claims.get("ga4gh_passport_v1").and_then(|v| v.as_array()) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for entry in passport {
            let Some(token) = entry.as_str() else {
                continue;
            };
            match verify_visa_jwt(token, keys) {
                Ok(visa) => out.push(visa),
                Err(err) => {
                    tracing::warn!(error = %err, "skipping unverified or invalid passport visa");
                }
            }
        }
        out
    }

    pub fn has_controlled_grant_for_dataset(visas: &[PassportVisa], dataset_id: &str) -> bool {
        visas.iter().any(|v| {
            v.visa_type == VisaType::ControlledAccessGrants && dataset_matches(&v.value, dataset_id)
        })
    }
}

/// Exact string match, or last path segment of a URL equal to `dataset_id` (not substring).
pub fn dataset_matches(visa_value: &str, dataset_id: &str) -> bool {
    if visa_value == dataset_id {
        return true;
    }
    if let Ok(u) = Url::parse(visa_value) {
        if let Some(seg) = u.path_segments().and_then(|mut s| s.next_back()) {
            return seg == dataset_id;
        }
    }
    false
}

/// Three-tier Beacon mapping plus deny for controlled-without-grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaconAccessTier {
    Public,
    Registered,
    Controlled,
    /// Controlled dataset, no verified grant — fail closed.
    Denied,
}

pub fn access_tier_for_beacon(
    cfg_level: BeaconAccessLevel,
    claims: Option<&Value>,
    dataset_id: &str,
) -> BeaconAccessTier {
    // No implicit JWKS fetch. Callers that need visa HTTP must pass a key source
    // with an issuer allowlist.
    access_tier_for_beacon_with(cfg_level, claims, dataset_id, &HttpJwks::new())
}

pub fn access_tier_for_beacon_with(
    cfg_level: BeaconAccessLevel,
    claims: Option<&Value>,
    dataset_id: &str,
    keys: &dyn VisaKeySource,
) -> BeaconAccessTier {
    match cfg_level {
        BeaconAccessLevel::Public => BeaconAccessTier::Public,
        BeaconAccessLevel::Registered => {
            if claims.is_some() {
                BeaconAccessTier::Registered
            } else {
                BeaconAccessTier::Public
            }
        }
        BeaconAccessLevel::Controlled => {
            let Some(c) = claims else {
                return BeaconAccessTier::Denied;
            };
            let visas = VisaEvaluator::visas_from_claims_with(c, keys);
            if VisaEvaluator::has_controlled_grant_for_dataset(&visas, dataset_id) {
                BeaconAccessTier::Controlled
            } else {
                BeaconAccessTier::Denied
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde_json::json;

    const TEST_HS: &[u8] = b"lab-kit-auth-visa-test-secret-32b!";

    struct HmacKeys;

    impl VisaKeySource for HmacKeys {
        fn decoding_key(
            &self,
            issuer: &str,
            kid: &str,
            _header_alg: Algorithm,
        ) -> Result<DecodingKey, AuthError> {
            if issuer != "https://visas.test" || kid != "test-kid" {
                return Err(AuthError::Oidc("unexpected iss/kid".into()));
            }
            Ok(DecodingKey::from_secret(TEST_HS))
        }
    }

    fn signed_visa(dataset: &str) -> String {
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("test-kid".into());
        let claims = json!({
            "iss": "https://visas.test",
            "sub": "user-1",
            "exp": 4_102_444_800_u64,
            "ga4gh_visa_v1": {
                "type": "ControlledAccessGrants",
                "value": dataset,
                "source": "https://dac.test",
                "by": "dac"
            }
        });
        encode(&header, &claims, &EncodingKey::from_secret(TEST_HS)).unwrap()
    }

    #[test]
    fn parses_ga4gh_visa_v1_shape() {
        let payload = json!({
            "iss": "https://visas.test",
            "ga4gh_visa_v1": {
                "type": "ControlledAccessGrants",
                "value": "https://example.org/datasets/ds1",
                "by": "dac"
            }
        });
        let v = visa_from_verified_payload(&payload).unwrap();
        assert_eq!(v.visa_type, VisaType::ControlledAccessGrants);
        assert_eq!(v.value, "https://example.org/datasets/ds1");
        assert_eq!(v.asserted_by, "dac");
    }

    #[test]
    fn dataset_match_is_not_substring() {
        assert!(dataset_matches("ds1", "ds1"));
        assert!(!dataset_matches("ds10", "ds1"));
        assert!(dataset_matches("https://example.org/datasets/ds1", "ds1"));
        assert!(!dataset_matches("https://example.org/datasets/ds10", "ds1"));
    }

    #[test]
    fn unverified_or_forged_visa_is_skipped() {
        let claims = json!({
            "ga4gh_passport_v1": ["not-a-jwt", "aaa.bbb.ccc"]
        });
        let visas = VisaEvaluator::visas_from_claims_with(&claims, &HmacKeys);
        assert!(visas.is_empty());
    }

    #[test]
    fn verified_visa_grants_controlled_tier() {
        let token = signed_visa("https://example.org/datasets/cohort-1");
        let visa = verify_visa_jwt(&token, &HmacKeys).expect("verify visa jwt");
        assert_eq!(visa.visa_type, VisaType::ControlledAccessGrants);
        let claims = json!({ "ga4gh_passport_v1": [token] });
        let tier = access_tier_for_beacon_with(
            BeaconAccessLevel::Controlled,
            Some(&claims),
            "cohort-1",
            &HmacKeys,
        );
        assert_eq!(tier, BeaconAccessTier::Controlled);
    }

    #[test]
    fn wrong_dataset_is_denied() {
        let token = signed_visa("https://example.org/datasets/other");
        let claims = json!({ "ga4gh_passport_v1": [token] });
        let tier = access_tier_for_beacon_with(
            BeaconAccessLevel::Controlled,
            Some(&claims),
            "cohort-1",
            &HmacKeys,
        );
        assert_eq!(tier, BeaconAccessTier::Denied);
    }

    #[test]
    fn http_jwks_without_allowlist_does_not_fetch() {
        let err = HttpJwks::new()
            .decoding_key("https://evil.example", "kid", Algorithm::RS256)
            .unwrap_err();
        assert!(err.to_string().contains("allowlist"));
    }

    #[test]
    fn old_flat_visa_shape_is_rejected() {
        let payload = json!({
            "visa_type": "controlled_access_grants",
            "asserted_by": "dac",
            "value": "ds1"
        });
        assert!(visa_from_verified_payload(&payload).is_none());
    }
}
