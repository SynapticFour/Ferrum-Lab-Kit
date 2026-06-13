//! Canonical `lab-kit.toml` schema (see repository `config/lab-kit.example.toml`).

use serde::{Deserialize, Serialize};
use url::Url;

use crate::CoreError;

/// Root configuration: only sections a lab actually uses need to be filled in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabKitConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub lab: LabSection,
    pub auth: AuthSection,
    #[serde(default)]
    pub services: ServicesSection,
    #[serde(default)]
    pub external: ExternalSection,
    /// Optional **ferrum-gateway** base URL for CLI helpers (e.g. `lab-kit ingest`).
    #[serde(default)]
    pub ferrum: FerrumSection,
    /// Set when generated from a deployment profile (e.g. field-edge).
    #[serde(default)]
    pub meta: Option<MetaSection>,
    #[serde(default)]
    pub backend: Option<BackendSection>,
    #[serde(default)]
    pub africa: Option<ProfileAfricaSection>,
    #[serde(default)]
    pub network: Option<ProfileNetworkSection>,
    #[serde(default)]
    pub resources: Option<ProfileResourcesSection>,
    #[serde(default)]
    pub conformance: Option<ConformanceSection>,
    /// Optional co-deployed **ga4gh-infra** auth plane (broker, visa-registry, ADS, …).
    #[serde(default)]
    pub ga4gh_infra: Option<Ga4ghInfraSection>,
}

fn default_schema_version() -> u32 {
    1
}

impl LabKitConfig {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.schema_version != 1 {
            return Err(CoreError::Validation(format!(
                "unsupported schema_version {} (expected 1)",
                self.schema_version
            )));
        }

        match self.auth.provider {
            AuthProvider::LsLogin => {
                if self.auth.ls_login.is_none() {
                    return Err(CoreError::Validation(
                        "auth.provider = \"ls-login\" requires [auth.ls-login]".into(),
                    ));
                }
            }
            AuthProvider::Keycloak => {
                if self.auth.keycloak.is_none() {
                    return Err(CoreError::Validation(
                        "auth.provider = \"keycloak\" requires [auth.keycloak]".into(),
                    ));
                }
            }
            AuthProvider::Ldap => {
                if self.auth.ldap.is_none() {
                    return Err(CoreError::Validation(
                        "auth.provider = \"ldap\" requires [auth.ldap]".into(),
                    ));
                }
            }
            AuthProvider::Local | AuthProvider::None => {}
        }

        if !self.services.any_configured() && self.external.is_empty() {
            return Err(CoreError::Validation(
                "configure at least one [services.*] block and/or [external] endpoint".into(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabSection {
    pub name: String,
    #[serde(default)]
    pub contact: Option<String>,
    pub environment: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthProvider {
    #[serde(rename = "ls-login")]
    LsLogin,
    Keycloak,
    Ldap,
    /// Local Passport validation (offline-capable; no external IdP).
    Local,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSection {
    pub provider: AuthProvider,
    #[serde(rename = "ls-login", default)]
    pub ls_login: Option<LsLoginConfig>,
    #[serde(default)]
    pub keycloak: Option<KeycloakConfig>,
    #[serde(default)]
    pub ldap: Option<LdapAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsLoginConfig {
    pub client_id: String,
    pub client_secret: String,
    /// Override discovery; default ELIXIR Czech broker.
    #[serde(default = "default_ls_login_issuer")]
    pub issuer: String,
    #[serde(default)]
    pub redirect_uri: Option<Url>,
    #[serde(default)]
    pub scopes: Vec<String>,
}

pub fn default_ls_login_issuer() -> String {
    "https://login.elixir-czech.org/oidc/".to_string()
}

/// Deployment profile metadata (from `config/profiles/*.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSection {
    pub profile: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub target_hardware: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendSection {
    #[serde(default = "default_backend_database")]
    pub database: String,
    #[serde(default = "default_backend_storage")]
    pub storage: String,
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: String,
    #[serde(default = "default_objects_path")]
    pub objects_path: String,
}

fn default_backend_database() -> String {
    "sqlite".into()
}
fn default_backend_storage() -> String {
    "local-filesystem".into()
}
fn default_sqlite_path() -> String {
    "~/.ferrum/ferrum.db".into()
}
fn default_objects_path() -> String {
    "~/.ferrum/objects".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileAfricaSection {
    #[serde(default = "default_true")]
    pub offline_first: bool,
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u32,
    #[serde(default = "default_true")]
    pub power_monitor: bool,
    #[serde(default = "default_low_power")]
    pub low_power_threshold: u32,
    #[serde(default = "default_emergency")]
    pub emergency_threshold: u32,
}

fn default_true() -> bool {
    true
}
fn default_max_memory_mb() -> u32 {
    3072
}
fn default_low_power() -> u32 {
    40
}
fn default_emergency() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileAuthSection {
    #[serde(default = "default_auth_mode")]
    pub mode: String,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub issuer: Option<String>,
}

fn default_auth_mode() -> String {
    "local".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileNetworkSection {
    #[serde(default = "default_true")]
    pub bandwidth_adaptive: bool,
    #[serde(default = "default_sync_schedule")]
    pub opportunistic_sync_schedule: String,
    #[serde(default = "default_chunk_kb")]
    pub chunk_size_low_bandwidth_kb: u32,
}

fn default_sync_schedule() -> String {
    "0 2 * * *".into()
}
fn default_chunk_kb() -> u32 {
    512
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileResourcesSection {
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_requests: u32,
    #[serde(default)]
    pub background_indexing: bool,
}

fn default_max_concurrent() -> u32 {
    4
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConformanceSection {
    #[serde(default = "default_helixtest_timeout")]
    pub helixtest_timeout_seconds: u32,
}

fn default_helixtest_timeout() -> u32 {
    120
}

/// Boolean service flags in profile templates (distinct from `[services.*]` blocks).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileServicesFlags {
    #[serde(default)]
    pub beacon: bool,
    #[serde(default)]
    pub drs: bool,
    #[serde(default)]
    pub htsget: bool,
    #[serde(default)]
    pub wes: bool,
    #[serde(default)]
    pub tes: bool,
    #[serde(default)]
    pub trs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakConfig {
    pub issuer: Url,
    pub realm: String,
    pub client_id: String,
    #[serde(default)]
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapAuthConfig {
    pub url: Url,
    #[serde(default)]
    pub bind_dn: Option<String>,
    #[serde(default)]
    pub base_dn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServicesSection {
    #[serde(default)]
    pub drs: Option<DrsServiceConfig>,
    #[serde(default)]
    pub htsget: Option<HtsgetServiceConfig>,
    #[serde(default)]
    pub wes: Option<WesServiceConfig>,
    #[serde(default)]
    pub tes: Option<TesServiceConfig>,
    #[serde(default)]
    pub beacon: Option<BeaconServiceConfig>,
    #[serde(default)]
    pub trs: Option<TrsServiceConfig>,
}

impl ServicesSection {
    pub fn any_configured(&self) -> bool {
        self.drs.is_some()
            || self.htsget.is_some()
            || self.wes.is_some()
            || self.tes.is_some()
            || self.beacon.is_some()
            || self.trs.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DrsServiceConfig {
    #[serde(default)]
    pub external_url: Option<Url>,
    #[serde(default)]
    pub storage_backend: Option<String>,
    #[serde(default)]
    pub s3: Option<S3NestedConfig>,
    #[serde(default)]
    pub posix: Option<PosixNestedConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HtsgetServiceConfig {
    #[serde(default)]
    pub external_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WesServiceConfig {
    #[serde(default)]
    pub external_url: Option<Url>,
    #[serde(default)]
    pub workflow_engine: Option<String>,
    #[serde(default)]
    pub compute_backend: Option<String>,
    #[serde(default)]
    pub slurm: Option<SlurmNestedConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TesServiceConfig {
    #[serde(default)]
    pub external_url: Option<Url>,
    #[serde(default)]
    pub compute_backend: Option<String>,
    #[serde(default)]
    pub slurm: Option<SlurmNestedConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconServiceConfig {
    #[serde(default)]
    pub external_url: Option<Url>,
    pub dataset_id: String,
    #[serde(default = "default_beacon_access")]
    pub access_level: BeaconAccessLevel,
}

fn default_beacon_access() -> BeaconAccessLevel {
    BeaconAccessLevel::Registered
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BeaconAccessLevel {
    Public,
    Registered,
    Controlled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrsServiceConfig {
    #[serde(default)]
    pub external_url: Option<Url>,
    #[serde(default)]
    pub registry_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3NestedConfig {
    pub endpoint: Url,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default)]
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosixNestedConfig {
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurmNestedConfig {
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub partition: Option<String>,
}

/// Optional Ferrum gateway hints (not required for `generate compose` unless you use ingest CLI).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FerrumSection {
    /// Base URL of **ferrum-gateway** (no path), e.g. `http://localhost:8080`.
    #[serde(default)]
    pub gateway_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExternalSection {
    #[serde(default)]
    pub htsget_url: Option<Url>,
    #[serde(default)]
    pub beacon_network_url: Option<Url>,
}

impl ExternalSection {
    pub fn is_empty(&self) -> bool {
        self.htsget_url.is_none() && self.beacon_network_url.is_none()
    }
}

/// Co-deploy or external **ga4gh-infra** integration (see `deploy/docker-compose/infra.yml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ga4ghInfraSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub mode: Ga4ghInfraMode,
    /// When true, use SQLite-backed ga4gh-infra images (lighter / field-edge stacks).
    #[serde(default)]
    pub africa: bool,
    #[serde(default = "default_broker_port")]
    pub broker_port: u16,
    /// External service-registry base URL (no trailing slash). Used when `mode = "external"`.
    #[serde(default)]
    pub service_registry_url: Option<String>,
    /// Environment variable holding the registry registration API key.
    #[serde(default = "default_registration_api_key_env")]
    pub registration_api_key_env: String,
}

fn default_broker_port() -> u16 {
    8180
}

fn default_registration_api_key_env() -> String {
    "SERVICE_REGISTRY_REGISTRATION_KEY".into()
}

impl Default for Ga4ghInfraSection {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: Ga4ghInfraMode::Disabled,
            africa: false,
            broker_port: default_broker_port(),
            service_registry_url: None,
            registration_api_key_env: default_registration_api_key_env(),
        }
    }
}

/// How Ferrum Lab Kit wires ga4gh-infra relative to Ferrum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Ga4ghInfraMode {
    /// Do not deploy or configure ga4gh-infra.
    #[default]
    Disabled,
    /// Deploy ga4gh-infra alongside Ferrum (ports 8180–8190, mock-idp 9100).
    CoDeploy,
    /// Point Ferrum at an existing ga4gh-infra deployment (no `infra.yml` merge).
    External,
}

#[cfg(test)]
mod tests {
    #[test]
    fn minimal_beacon_ls_login() {
        let raw = r#"
schema_version = 1

[lab]
name = "Test Lab"
environment = "demo"

[auth]
provider = "ls-login"

[auth.ls-login]
client_id = "cid"
client_secret = "sec"
issuer = "https://login.elixir-czech.org/oidc/"

[services.beacon]
dataset_id = "ds1"
access_level = "registered"
"#;
        let c = crate::parse_config(raw).unwrap();
        assert_eq!(c.services.beacon.as_ref().unwrap().dataset_id, "ds1");
    }
}
