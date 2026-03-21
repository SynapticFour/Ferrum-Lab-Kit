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
            AuthProvider::None => {}
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

fn default_ls_login_issuer() -> String {
    "https://login.elixir-czech.org/oidc/".to_string()
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
