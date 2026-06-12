//! Deployment profile templates (e.g. `config/profiles/field-edge.toml`) and expansion
//! into canonical [`LabKitConfig`].

use serde::Deserialize;

use crate::config::{
    AuthProvider, AuthSection, BackendSection, BeaconAccessLevel, BeaconServiceConfig,
    ConformanceSection, DrsServiceConfig, HtsgetServiceConfig, LabKitConfig, LabSection,
    LsLoginConfig, MetaSection, PosixNestedConfig, ProfileAfricaSection, ProfileAuthSection,
    ProfileNetworkSection, ProfileResourcesSection, ProfileServicesFlags, ServicesSection,
    TesServiceConfig, TrsServiceConfig, WesServiceConfig,
};
use crate::CoreError;

/// Raw profile TOML (boolean service flags + edge metadata).
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileTemplate {
    pub meta: MetaSection,
    #[serde(default)]
    pub services: ProfileServicesFlags,
    #[serde(default)]
    pub backend: BackendSection,
    #[serde(default)]
    pub africa: ProfileAfricaSection,
    #[serde(default)]
    pub auth: ProfileAuthSection,
    #[serde(default)]
    pub network: ProfileNetworkSection,
    #[serde(default)]
    pub resources: ProfileResourcesSection,
    #[serde(default)]
    pub conformance: ConformanceSection,
}

impl ProfileTemplate {
    pub fn parse(raw: &str) -> Result<Self, CoreError> {
        toml::from_str(raw).map_err(CoreError::Toml)
    }

    /// Expand a profile template into a validated [`LabKitConfig`].
    pub fn into_lab_kit_config(
        self,
        lab_name: &str,
        environment: &str,
        dataset_id: &str,
        overrides: ProfileOverrides,
    ) -> Result<LabKitConfig, CoreError> {
        let mut africa = self.africa;
        if let Some(mb) = overrides.max_memory_mb {
            africa.max_memory_mb = mb;
        }

        let mut backend = self.backend;
        if let Some(dir) = overrides.data_dir {
            backend.objects_path = dir.clone();
            backend.sqlite_path = format!("{dir}/ferrum.db");
        }

        let auth = if let Some(ls) = overrides.ls_login.clone() {
            AuthSection {
                provider: AuthProvider::LsLogin,
                ls_login: Some(ls),
                keycloak: None,
                ldap: None,
            }
        } else {
            auth_from_profile(&self.auth, None)?
        };

        let mut services = ServicesSection::default();
        let svc = &self.services;
        if svc.beacon || overrides.enable_beacon {
            services.beacon = Some(BeaconServiceConfig {
                external_url: None,
                dataset_id: dataset_id.to_string(),
                access_level: BeaconAccessLevel::Registered,
            });
        }
        if svc.drs || overrides.enable_drs {
            services.drs = Some(DrsServiceConfig {
                external_url: None,
                storage_backend: Some("posix".into()),
                s3: None,
                posix: Some(PosixNestedConfig {
                    root: backend.objects_path.clone(),
                }),
            });
        }
        if svc.htsget || overrides.enable_htsget {
            services.htsget = Some(HtsgetServiceConfig::default());
        }
        if svc.wes || overrides.enable_wes {
            services.wes = Some(WesServiceConfig::default());
        }
        if svc.tes || overrides.enable_tes {
            services.tes = Some(TesServiceConfig::default());
        }
        if svc.trs || overrides.enable_trs {
            services.trs = Some(TrsServiceConfig::default());
        }

        let cfg = LabKitConfig {
            schema_version: 1,
            lab: LabSection {
                name: lab_name.to_string(),
                contact: None,
                environment: environment.to_string(),
            },
            auth,
            services,
            external: Default::default(),
            ferrum: Default::default(),
            meta: Some(self.meta),
            backend: Some(backend),
            africa: Some(africa),
            network: Some(self.network),
            resources: Some(self.resources),
            conformance: Some(self.conformance),
        };
        cfg.validate()?;
        Ok(cfg)
    }
}

/// Optional overrides when expanding a profile (interactive init or CLI flags).
#[derive(Debug, Clone, Default)]
pub struct ProfileOverrides {
    pub max_memory_mb: Option<u32>,
    pub data_dir: Option<String>,
    pub enable_beacon: bool,
    pub enable_drs: bool,
    pub enable_htsget: bool,
    pub enable_wes: bool,
    pub enable_tes: bool,
    pub enable_trs: bool,
    pub ls_login: Option<LsLoginConfig>,
}

fn auth_from_profile(
    auth: &ProfileAuthSection,
    ls_login: Option<LsLoginConfig>,
) -> Result<AuthSection, CoreError> {
    match auth.mode.as_str() {
        "local" => Ok(AuthSection {
            provider: AuthProvider::Local,
            ls_login: None,
            keycloak: None,
            ldap: None,
        }),
        "ls-login" => {
            let ls = ls_login.or_else(|| {
                Some(LsLoginConfig {
                    client_id: auth.client_id.clone().unwrap_or_default(),
                    client_secret: auth.client_secret.clone().unwrap_or_default(),
                    issuer: auth
                        .issuer
                        .clone()
                        .unwrap_or_else(crate::config::default_ls_login_issuer),
                    redirect_uri: None,
                    scopes: lab_kit_auth_scopes(),
                })
            });
            let ls = ls.filter(|c| !c.client_id.is_empty());
            if ls.is_none() {
                return Err(CoreError::Validation(
                    "auth.mode = \"ls-login\" requires client_id and client_secret".into(),
                ));
            }
            Ok(AuthSection {
                provider: AuthProvider::LsLogin,
                ls_login: ls,
                keycloak: None,
                ldap: None,
            })
        }
        other => Err(CoreError::Validation(format!(
            "unsupported auth.mode \"{other}\" in profile (expected \"local\" or \"ls-login\")"
        ))),
    }
}

fn lab_kit_auth_scopes() -> Vec<String> {
    vec![
        "openid".into(),
        "profile".into(),
        "email".into(),
        "offline_access".into(),
        "ga4gh_passport_v1".into(),
    ]
}

/// Load a profile template from `config/profiles/{name}.toml` relative to repo root or CWD.
pub fn load_profile_template(name: &str) -> Result<ProfileTemplate, CoreError> {
    let candidates = [
        format!("config/profiles/{name}.toml"),
        format!("../config/profiles/{name}.toml"),
    ];
    for path in &candidates {
        if let Ok(raw) = std::fs::read_to_string(path) {
            return ProfileTemplate::parse(&raw);
        }
    }
    Err(CoreError::Validation(format!(
        "profile \"{name}\" not found (looked for config/profiles/{name}.toml)"
    )))
}

/// Returns `true` when raw TOML is a profile template (has `[meta].profile`, no `[lab]`).
pub fn is_profile_template(raw: &str) -> bool {
    raw.contains("[meta]") && raw.contains("profile =") && !raw.contains("[lab]")
}

/// Parse either a canonical `lab-kit.toml` or a profile template.
pub fn parse_config_or_profile(raw: &str) -> Result<LabKitConfig, CoreError> {
    if is_profile_template(raw) {
        let template = ProfileTemplate::parse(raw)?;
        return template.into_lab_kit_config(
            "Field Lab",
            "production",
            "field-cohort-001",
            ProfileOverrides::default(),
        );
    }
    let cfg: LabKitConfig = toml::from_str(raw)?;
    cfg.validate()?;
    Ok(cfg)
}

/// Whether the config was generated from the field-edge profile.
pub fn is_field_edge(cfg: &LabKitConfig) -> bool {
    cfg.meta
        .as_ref()
        .map(|m| m.profile == "field-edge")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_edge_profile_expands() {
        let raw = include_str!("../../../config/profiles/field-edge.toml");
        let cfg = parse_config_or_profile(raw).unwrap();
        assert!(is_field_edge(&cfg));
        assert!(cfg.services.beacon.is_some());
        assert!(cfg.services.drs.is_some());
        assert!(cfg.services.wes.is_none());
        assert_eq!(cfg.auth.provider, AuthProvider::Local);
    }
}
