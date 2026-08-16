// SPDX-License-Identifier: BUSL-1.1
//! Signed license tokens for PDF reports.
//!
//! A valid key is `flk1.<base64url-json>.<base64url-ed25519-sig>` issued by
//! Synaptic Four. Arbitrary `flk_` strings are rejected.
//!
//! Verification uses (in order):
//! 1. `FERRUM_LAB_KIT_LICENSE_PUBKEY` (64 hex chars = 32-byte Ed25519 public key)
//! 2. file `crates/lab-kit-report/keys/license-ed25519.pub` or `$FERRUM_LAB_KIT_LICENSE_PUBKEY_FILE`
//!
//! There is no in-repo issuing secret. Generate a keypair with
//! `cargo run -p lab-kit-report --example gen_license_keypair`.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::sync::OnceLock;

use crate::ReportError;
use crate::LICENSE_KEY_ENV;

pub const LICENSE_FILE_ENV: &str = "FERRUM_LAB_KIT_LICENSE_FILE";
pub const LICENSE_PUBKEY_ENV: &str = "FERRUM_LAB_KIT_LICENSE_PUBKEY";
pub const LICENSE_PUBKEY_FILE_ENV: &str = "FERRUM_LAB_KIT_LICENSE_PUBKEY_FILE";

const TOKEN_PREFIX: &str = "flk1.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseActivation {
    pub key_hash: String,
    pub activated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub features: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    #[serde(default)]
    pub pdf: bool,
    #[serde(default)]
    pub exp: Option<String>,
    #[serde(default)]
    pub sub: Option<String>,
}

pub fn hash_license_key(key: &str) -> String {
    let mut h = Sha256::new();
    h.update(key.trim().as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn b64url_decode(s: &str) -> Result<Vec<u8>, ReportError> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|_| ReportError::License("license token is not base64url".into()))
}

fn parse_signed_token(key: &str) -> Result<(LicensePayload, [u8; 64]), ReportError> {
    let t = key.trim();
    let rest = t.strip_prefix(TOKEN_PREFIX).ok_or_else(|| {
        ReportError::License(format!(
            "invalid {LICENSE_KEY_ENV}: expected flk1.<payload>.<sig> signed by the vendor"
        ))
    })?;
    let mut parts = rest.splitn(2, '.');
    let payload_b64 = parts.next().unwrap_or("");
    let sig_b64 = parts
        .next()
        .ok_or_else(|| ReportError::License("license token missing signature component".into()))?;
    let payload_bytes = b64url_decode(payload_b64)?;
    let sig_bytes = b64url_decode(sig_b64)?;
    let sig: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| ReportError::License("license signature must be 64 bytes".into()))?;
    let payload: LicensePayload = serde_json::from_slice(&payload_bytes)?;
    Ok((payload, sig))
}

fn hex_decode_32(s: &str) -> Result<[u8; 32], ReportError> {
    let t = s.trim();
    if t.len() != 64 || !t.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ReportError::License(
            "license public key must be 64 hex characters".into(),
        ));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&t[i * 2..i * 2 + 2], 16)
            .map_err(|_| ReportError::License("invalid hex in license public key".into()))?;
    }
    Ok(out)
}

fn bundled_pubkey_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("keys/license-ed25519.pub")
}

fn load_verifying_key_bytes() -> Result<[u8; 32], ReportError> {
    if let Ok(hex) = std::env::var(LICENSE_PUBKEY_ENV) {
        if !hex.trim().is_empty() {
            return hex_decode_32(&hex);
        }
    }
    if let Ok(p) = std::env::var(LICENSE_PUBKEY_FILE_ENV) {
        if !p.trim().is_empty() {
            let raw = std::fs::read_to_string(p.trim()).map_err(|_| {
                ReportError::License(format!("cannot read {LICENSE_PUBKEY_FILE_ENV}"))
            })?;
            return hex_decode_32(&raw);
        }
    }
    let bundled = bundled_pubkey_path();
    if bundled.is_file() {
        let raw = std::fs::read_to_string(&bundled)
            .map_err(|_| ReportError::License("cannot read bundled license public key".into()))?;
        let line = raw
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && !l.starts_with('#'))
            .unwrap_or("");
        if !line.is_empty() {
            return hex_decode_32(line);
        }
    }
    Err(ReportError::License(format!(
        "no vendor Ed25519 public key configured — set {LICENSE_PUBKEY_ENV} or provide keys/license-ed25519.pub"
    )))
}

fn verifying_key() -> Result<VerifyingKey, ReportError> {
    #[cfg(test)]
    if let Some(k) = test_verifying_key() {
        return Ok(k);
    }
    let bytes = load_verifying_key_bytes()?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|_| ReportError::License("license public key is not a valid Ed25519 key".into()))
}

#[cfg(test)]
fn test_verifying_key() -> Option<VerifyingKey> {
    TEST_VK.get().copied()
}

#[cfg(test)]
static TEST_VK: OnceLock<VerifyingKey> = OnceLock::new();

#[cfg(test)]
pub fn install_test_verifying_key(vk: VerifyingKey) {
    let _ = TEST_VK.set(vk);
}

/// Verify signature, `pdf` feature, and expiry. Does not consult the activation file.
pub fn verify_signed_license(key: &str) -> Result<LicensePayload, ReportError> {
    let (payload, sig_bytes) = parse_signed_token(key)?;
    let t = key.trim();
    let payload_section = t
        .rsplit_once('.')
        .map(|(left, _)| left)
        .ok_or_else(|| ReportError::License("malformed license token".into()))?;
    let vk = verifying_key()?;
    let sig = Signature::from_bytes(&sig_bytes);
    vk.verify(payload_section.as_bytes(), &sig)
        .map_err(|_| ReportError::License("license signature is invalid".into()))?;
    if !payload.pdf {
        return Err(ReportError::License(
            "license token does not include pdf".into(),
        ));
    }
    if let Some(exp) = payload.exp.as_deref() {
        let dt = DateTime::parse_from_rfc3339(exp).map_err(|_| {
            ReportError::License("license exp is not RFC3339 — refusing to grant".into())
        })?;
        if dt.with_timezone(&Utc) < Utc::now() {
            return Err(ReportError::License("license token has expired".into()));
        }
    }
    Ok(payload)
}

/// Legacy helper: signed tokens are well-formed; unsigned `flk_` blobs are not.
pub fn license_key_well_formed(key: &str) -> bool {
    parse_signed_token(key).is_ok()
}

pub fn default_license_file() -> PathBuf {
    if let Ok(p) = std::env::var(LICENSE_FILE_ENV) {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".ferrum/license-activation.json")
}

pub fn activate_license(
    key: &str,
    path: &Path,
    expires_at: Option<DateTime<Utc>>,
) -> Result<LicenseActivation, ReportError> {
    let payload = verify_signed_license(key)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let rec = LicenseActivation {
        key_hash: hash_license_key(key),
        activated_at: Utc::now().to_rfc3339(),
        expires_at: expires_at.map(|t| t.to_rfc3339()).or(payload.exp.clone()),
        features: serde_json::json!({ "pdf": true }),
    };
    std::fs::write(path, serde_json::to_string_pretty(&rec)?)?;
    Ok(rec)
}

pub fn pdf_license_granted(key: &str, activation_path: &Path) -> Result<(), ReportError> {
    let payload = verify_signed_license(key)?;
    let raw = std::fs::read_to_string(activation_path).map_err(|_| {
        ReportError::License(format!(
            "no license activation at {} — run `lab-kit license activate`",
            activation_path.display()
        ))
    })?;
    let rec: LicenseActivation = serde_json::from_str(&raw)?;
    if rec.key_hash != hash_license_key(key) {
        return Err(ReportError::License(
            "license key does not match the activation file".into(),
        ));
    }
    if let Some(exp) = rec.expires_at.as_deref().or(payload.exp.as_deref()) {
        let dt = DateTime::parse_from_rfc3339(exp).map_err(|_| {
            ReportError::License("activation expires_at is not RFC3339 — refusing to grant".into())
        })?;
        if dt.with_timezone(&Utc) < Utc::now() {
            return Err(ReportError::License(
                "license activation has expired".into(),
            ));
        }
    }
    if rec.features.get("pdf").and_then(|v| v.as_bool()) == Some(false) {
        return Err(ReportError::License(
            "activation does not include pdf".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
pub fn mint_test_token(pdf: bool, exp: Option<&str>) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use ed25519_dalek::Signer;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NONCE: AtomicU64 = AtomicU64::new(0);
    let seed = [0x11u8; 32];
    let sk = ed25519_dalek::SigningKey::from_bytes(&seed);
    install_test_verifying_key(sk.verifying_key());
    let nonce = NONCE.fetch_add(1, Ordering::SeqCst);
    let payload = serde_json::json!({ "pdf": pdf, "exp": exp, "sub": format!("test-{nonce}") });
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
    let signing_input = format!("{TOKEN_PREFIX}{payload_b64}");
    let sig = sk.sign(signing_input.as_bytes());
    format!("{signing_input}.{}", URL_SAFE_NO_PAD.encode(sig.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsigned_flk_blobs() {
        assert!(!license_key_well_formed("x"));
        assert!(!license_key_well_formed(&format!("flk_{}", "a".repeat(32))));
        verify_signed_license(&format!("flk_{}", "b".repeat(32))).unwrap_err();
    }

    #[test]
    fn activate_and_grant_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lic.json");
        let key = mint_test_token(true, None);
        activate_license(&key, &path, None).unwrap();
        pdf_license_granted(&key, &path).unwrap();
        pdf_license_granted(&mint_test_token(true, None), &path).unwrap_err();
    }

    #[test]
    fn malformed_expiry_is_denied() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lic.json");
        let key = mint_test_token(true, None);
        activate_license(&key, &path, None).unwrap();
        let mut rec: LicenseActivation =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        rec.expires_at = Some("not-a-date".into());
        std::fs::write(&path, serde_json::to_string(&rec).unwrap()).unwrap();
        let err = pdf_license_granted(&key, &path).unwrap_err();
        assert!(err.to_string().contains("RFC3339"));
    }

    #[test]
    fn pdf_false_is_denied() {
        let key = mint_test_token(false, None);
        verify_signed_license(&key).unwrap_err();
    }
}
