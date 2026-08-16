// SPDX-License-Identifier: BUSL-1.1
//! Deployment profile templates (e.g. `config/profiles/field-edge.toml`) and expansion
//! into canonical [`LabKitConfig`].

use serde::Deserialize;

use crate::config::{
    AuthProvider, AuthSection, BackendSection, BeaconAccessLevel, BeaconServiceConfig, BraSection,
    ConformanceSection, DrsServiceConfig, Ga4ghInfraMode, Ga4ghInfraSection, HtsgetServiceConfig,
    LabKitConfig, LabSection, LsLoginConfig, MetaSection, PosixNestedConfig, ProfileAfricaSection,
    ProfileAuthSection, ProfileNetworkSection, ProfileResourcesSection, ProfileServicesFlags,
    ServicesSection, SolumSection, TesServiceConfig, TrsServiceConfig, WesServiceConfig,
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
    #[serde(default)]
    pub ga4gh_infra: Ga4ghInfraSection,
    #[serde(default)]
    pub solum: SolumSection,
    #[serde(default)]
    pub bra: BraSection,
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
            ga4gh_infra: ga4gh_infra_from_profile(&self.ga4gh_infra),
            solum: solum_from_profile(&self.solum),
            bra: bra_from_profile(&self.bra),
        };
        cfg.validate()?;
        Ok(cfg)
    }
}

fn solum_from_profile(section: &SolumSection) -> Option<SolumSection> {
    if section.enabled {
        Some(section.clone())
    } else {
        None
    }
}

fn bra_from_profile(section: &BraSection) -> Option<BraSection> {
    if section.enabled {
        Some(section.clone())
    } else {
        None
    }
}

fn ga4gh_infra_from_profile(section: &Ga4ghInfraSection) -> Option<Ga4ghInfraSection> {
    if section.enabled && section.mode != Ga4ghInfraMode::Disabled {
        Some(section.clone())
    } else {
        None
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
            let mut ls = ls_login.or_else(|| {
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
            if let Some(c) = ls.as_mut() {
                if c.client_secret.trim().is_empty() {
                    c.client_secret = crate::config::resolve_oidc_client_secret(
                        "",
                        crate::config::LS_LOGIN_CLIENT_SECRET_ENV,
                    );
                }
                if c.client_secret.trim().is_empty() && !c.client_id.trim().is_empty() {
                    c.client_secret = crate::config::oidc_client_secret_placeholder();
                }
            }
            let ls =
                ls.filter(|c| !c.client_id.trim().is_empty() && !c.client_secret.trim().is_empty());
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

/// Load a profile template from the binary (shipped profiles) or `config/profiles/{name}.toml`.
pub fn load_profile_template(name: &str) -> Result<ProfileTemplate, CoreError> {
    if let Some(raw) = bundled_profile_toml(name) {
        if let Ok(t) = ProfileTemplate::parse(raw) {
            return Ok(t);
        }
    }
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
        "profile \"{name}\" not found (embedded or config/profiles/{name}.toml)"
    )))
}

/// Parse a named bundled/on-disk document as either a profile template or canonical `lab-kit.toml`.
pub fn load_named_config(name: &str) -> Result<LabKitConfig, CoreError> {
    if let Some(raw) = bundled_profile_toml(name) {
        return parse_config_or_profile(raw);
    }
    let candidates = [
        format!("config/profiles/{name}.toml"),
        format!("../config/profiles/{name}.toml"),
    ];
    for path in &candidates {
        if let Ok(raw) = std::fs::read_to_string(path) {
            return parse_config_or_profile(&raw);
        }
    }
    Err(CoreError::Validation(format!(
        "profile \"{name}\" not found (embedded or config/profiles/{name}.toml)"
    )))
}

/// Shipped profile/config TOML compiled into the CLI.
pub fn bundled_profile_toml(name: &str) -> Option<&'static str> {
    match name {
        "field-edge" => Some(include_str!("../../../config/profiles/field-edge.toml")),
        "field-edge+infra" => Some(include_str!(
            "../../../config/profiles/field-edge+infra.toml"
        )),
        "field-edge+solum" => Some(include_str!(
            "../../../config/profiles/field-edge+solum.toml"
        )),
        "field-edge+infra+solum" => Some(include_str!(
            "../../../config/profiles/field-edge+infra+solum.toml"
        )),
        "institute" => Some(include_str!("../../../config/profiles/institute.toml")),
        "full-elixir-node" => Some(include_str!(
            "../../../config/profiles/full-elixir-node.toml"
        )),
        "gdi-national-node" => Some(include_str!(
            "../../../config/profiles/gdi-national-node.toml"
        )),
        "beacon-only" => Some(include_str!("../../../config/profiles/beacon-only.toml")),
        "drs-wes" => Some(include_str!("../../../config/profiles/drs-wes.toml")),
        "bra-companion" => Some(include_str!("../../../config/profiles/bra-companion.toml")),
        "archive-submitter" => Some(include_str!(
            "../../../config/profiles/archive-submitter.toml"
        )),
        _ => None,
    }
}

/// Returns `true` when raw TOML is a profile template (`[meta].profile` present, no `[lab]`).
pub fn is_profile_template(raw: &str) -> bool {
    let Ok(v) = raw.parse::<toml::Value>() else {
        return false;
    };
    v.get("meta")
        .and_then(|m| m.get("profile"))
        .and_then(|p| p.as_str())
        .is_some()
        && v.get("lab").is_none()
}

/// Parse either a canonical `lab-kit.toml` or a profile template.
pub fn parse_config_or_profile(raw: &str) -> Result<LabKitConfig, CoreError> {
    if is_profile_template(raw) {
        let template = ProfileTemplate::parse(raw)?;
        let environment = if template.meta.profile.starts_with("field-edge") {
            "field"
        } else {
            "production"
        };
        return template.into_lab_kit_config(
            "Field Lab",
            environment,
            "field-cohort-001",
            ProfileOverrides::default(),
        );
    }
    let cfg: LabKitConfig = toml::from_str(raw)?;
    cfg.validate()?;
    Ok(cfg)
}

/// Whether the config was generated from a field-edge family profile.
pub fn is_field_edge(cfg: &LabKitConfig) -> bool {
    cfg.meta
        .as_ref()
        .map(|m| {
            matches!(
                m.profile.as_str(),
                "field-edge"
                    | "field-edge+infra"
                    | "field-edge+solum"
                    | "field-edge+infra+solum"
                    | "archive-submitter"
            )
        })
        .unwrap_or(false)
}

/// Archive-submitter: edge surfaces + Metadata Store (no WES/TES).
pub fn is_archive_submitter(cfg: &LabKitConfig) -> bool {
    cfg.meta
        .as_ref()
        .is_some_and(|m| m.profile == "archive-submitter")
}

/// Whether Ferrum and ga4gh-infra are co-deployed on the same host (local infra stack).
pub fn is_co_deploy(cfg: &LabKitConfig) -> bool {
    cfg.ga4gh_infra
        .as_ref()
        .is_some_and(|g| g.enabled && g.mode == Ga4ghInfraMode::CoDeploy)
}

/// Whether Solum sidecar co-deploy is enabled in config.
pub fn is_solum_enabled(cfg: &LabKitConfig) -> bool {
    cfg.solum.as_ref().is_some_and(|s| s.enabled)
}

/// Whether BRA workbench co-deploy is enabled in config.
pub fn is_bra_enabled(cfg: &LabKitConfig) -> bool {
    cfg.bra.as_ref().is_some_and(|s| s.enabled)
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

    #[test]
    fn field_edge_infra_profile_is_co_deploy() {
        let raw = include_str!("../../../config/profiles/field-edge+infra.toml");
        let cfg = parse_config_or_profile(raw).unwrap();
        assert!(is_field_edge(&cfg));
        assert!(is_co_deploy(&cfg));
        assert!(cfg.ga4gh_infra.as_ref().is_some_and(|g| g.enabled));
    }

    #[test]
    fn bra_companion_profile_enables_bra_and_wes() {
        let raw = include_str!("../../../config/profiles/bra-companion.toml");
        let cfg = parse_config_or_profile(raw).unwrap();
        assert!(is_bra_enabled(&cfg));
        assert!(cfg.services.drs.is_some());
        assert!(cfg.services.wes.is_some());
        assert_eq!(cfg.bra.as_ref().unwrap().bra_tag, "v0.2.0");
    }

    #[test]
    fn archive_submitter_is_edge_without_wes() {
        let raw = include_str!("../../../config/profiles/archive-submitter.toml");
        let cfg = parse_config_or_profile(raw).unwrap();
        assert!(is_archive_submitter(&cfg));
        assert!(is_field_edge(&cfg));
        assert!(cfg.services.drs.is_some());
        assert!(cfg.services.wes.is_none());
        assert!(cfg.services.tes.is_none());
        assert!(cfg.services.trs.is_none());
    }
}
