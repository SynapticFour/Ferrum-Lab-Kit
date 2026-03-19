//! Row types for Lab Kit metadata tables (`service_registry`, `conformance_runs`, `license_activations`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One row in `service_registry`: which GA4GH surface is active and how to reach it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistryRow {
    pub lab_name: String,
    pub service_name: String,
    pub endpoint_url: Option<String>,
    pub health_ok: Option<bool>,
    pub last_health_check: Option<DateTime<Utc>>,
}

/// Input for inserting a conformance run (database assigns `id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceRunInsert {
    pub helix_output: Value,
    pub overall_pass: bool,
    pub per_service: Value,
}

/// Full conformance run row including database id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceRunRow {
    pub id: i64,
    pub run_at: DateTime<Utc>,
    pub helix_output: Value,
    pub overall_pass: bool,
    pub per_service: Value,
}

/// License activation record (hashed key, features JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseActivationRow {
    pub key_hash: String,
    pub activated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub features: Value,
}
