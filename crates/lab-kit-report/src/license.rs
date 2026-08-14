//! License-key hashing and activation for PDF reports.
//!
//! The env var `FERRUM_LAB_KIT_LICENSE_KEY` alone is not enough: the key must be
//! well-formed (`flk_` + ≥32 unreserved chars) and match a previously activated
//! hash file (see `lab-kit license activate`).

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ReportError;
use crate::LICENSE_KEY_ENV;

/// Default activation document (`$HOME/.ferrum/license-activation.json`).
pub const LICENSE_FILE_ENV: &str = "FERRUM_LAB_KIT_LICENSE_FILE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseActivation {
    pub key_hash: String,
    pub activated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub features: serde_json::Value,
}

pub fn hash_license_key(key: &str) -> String {
    let mut h = Sha256::new();
    h.update(key.trim().as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// `flk_` prefix plus at least 32 alphanumeric / `_` / `-` characters.
pub fn license_key_well_formed(key: &str) -> bool {
    let t = key.trim();
    let Some(rest) = t.strip_prefix("flk_") else {
        return false;
    };
    rest.len() >= 32
        && rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
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
    if !license_key_well_formed(key) {
        return Err(ReportError::License(
            "key must look like flk_<32+ unreserved chars>".into(),
        ));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let rec = LicenseActivation {
        key_hash: hash_license_key(key),
        activated_at: Utc::now().to_rfc3339(),
        expires_at: expires_at.map(|t| t.to_rfc3339()),
        features: serde_json::json!({ "pdf": true }),
    };
    std::fs::write(path, serde_json::to_string_pretty(&rec)?)?;
    Ok(rec)
}

pub fn pdf_license_granted(key: &str, activation_path: &Path) -> Result<(), ReportError> {
    if !license_key_well_formed(key) {
        return Err(ReportError::License(format!(
            "invalid {LICENSE_KEY_ENV}: expected flk_<32+ chars>. Run `lab-kit license activate`."
        )));
    }
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
    if let Some(exp) = rec.expires_at.as_deref() {
        if let Ok(dt) = DateTime::parse_from_rfc3339(exp) {
            if dt.with_timezone(&Utc) < Utc::now() {
                return Err(ReportError::License(
                    "license activation has expired".into(),
                ));
            }
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
mod tests {
    use super::*;

    #[test]
    fn rejects_short_or_unprefixed_keys() {
        assert!(!license_key_well_formed("x"));
        assert!(!license_key_well_formed("flk_short"));
        assert!(license_key_well_formed(&format!("flk_{}", "a".repeat(32))));
    }

    #[test]
    fn activate_and_grant_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lic.json");
        let key = format!("flk_{}", "b".repeat(32));
        activate_license(&key, &path, None).unwrap();
        pdf_license_granted(&key, &path).unwrap();
        pdf_license_granted("flk_cccccccccccccccccccccccccccccccc", &path).unwrap_err();
    }
}
