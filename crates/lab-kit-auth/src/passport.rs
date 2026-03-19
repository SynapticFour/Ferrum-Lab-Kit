//! GA4GH Passport claim handling (`ga4gh_passport_v1`) and Beacon v2 access tiers.

use lab_kit_core::BeaconAccessLevel;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// High-level visa types referenced by Beacon controlled access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisaType {
    ControlledAccessGrants,
    ResearcherStatus,
    AffiliationAndRole,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportVisa {
    pub visa_type: VisaType,
    pub asserted_by: String,
    pub value: String,
}

pub struct VisaEvaluator;

impl VisaEvaluator {
    /// Decode passport array from OIDC claims (each entry is a nested JWT string per GA4GH AAI).
    pub fn visas_from_claims(claims: &Value) -> Vec<PassportVisa> {
        let Some(passport) = claims.get("ga4gh_passport_v1").and_then(|v| v.as_array()) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for entry in passport {
            let Some(token) = entry.as_str() else {
                continue;
            };
            if let Ok(payload) = decode_passport_jwt_unverified(token) {
                if let Ok(visa) = serde_json::from_value::<PassportVisa>(payload) {
                    out.push(visa);
                }
            }
        }
        out
    }

    pub fn has_controlled_grant_for_dataset(visas: &[PassportVisa], dataset_id: &str) -> bool {
        visas.iter().any(|v| {
            v.visa_type == VisaType::ControlledAccessGrants && v.value.contains(dataset_id)
        })
    }
}

fn decode_passport_jwt_unverified(token: &str) -> Result<Value, ()> {
    let mut parts = token.split('.');
    let _h = parts.next().ok_or(())?;
    let p = parts.next().ok_or(())?;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let bytes = URL_SAFE_NO_PAD.decode(p).map_err(|_| ())?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|_| ())?;
    Ok(v)
}

/// Three-tier Beacon mapping: public / registered / controlled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaconAccessTier {
    Public,
    Registered,
    Controlled,
}

pub fn access_tier_for_beacon(
    cfg_level: BeaconAccessLevel,
    claims: Option<&Value>,
    dataset_id: &str,
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
                return BeaconAccessTier::Public;
            };
            let visas = VisaEvaluator::visas_from_claims(c);
            if VisaEvaluator::has_controlled_grant_for_dataset(&visas, dataset_id) {
                BeaconAccessTier::Controlled
            } else if claims.is_some() {
                BeaconAccessTier::Registered
            } else {
                BeaconAccessTier::Public
            }
        }
    }
}
