use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use lab_kit_core::{LabKitConfig, ServiceId, ServiceRegistry};
use serde::Serialize;

use crate::DeployError;

/// Sidecar metadata for reverse proxies when services use `external_url` (bring-your-own).
#[derive(Debug, Serialize)]
struct ExternalUpstreamsFile {
    /// How to use: point Traefik/Caddy/Envoy at these URLs for the given GA4GH routes.
    note: &'static str,
    externals: Vec<ExternalUpstream>,
}

#[derive(Debug, Serialize)]
struct ExternalUpstream {
    service: String,
    base_url: String,
}

/// Write `external-upstreams.yaml` next to the compose file when any service is external.
pub fn write_external_upstreams_next_to_compose(
    cfg: &LabKitConfig,
    compose_output: &Path,
) -> Result<(), DeployError> {
    let registry = ServiceRegistry::from_config(cfg);
    let mut externals = Vec::new();
    for e in &registry.entries {
        if let Some(url) = &e.external_base {
            externals.push(ExternalUpstream {
                service: format!("{:?}", e.id).to_lowercase(),
                base_url: url.to_string(),
            });
        }
    }

    if externals.is_empty() {
        return Ok(());
    }

    let doc = ExternalUpstreamsFile {
        note: "Services listed here are not deployed by the merged docker-compose; route your gateway to base_url. See docs/BRING-YOUR-OWN.md.",
        externals,
    };

    let out_path: PathBuf = compose_output
        .parent()
        .unwrap_or(Path::new("."))
        .join("external-upstreams.yaml");
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, serde_yaml::to_string(&doc)?)?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct TraefikDynamicConfig {
    http: TraefikHttp,
}

#[derive(Debug, Serialize)]
struct TraefikHttp {
    routers: BTreeMap<String, TraefikRouter>,
    services: BTreeMap<String, TraefikService>,
}

#[derive(Debug, Serialize)]
struct TraefikRouter {
    /// Example: `PathPrefix(`/ga4gh/wes/v1`)`
    rule: String,
    service: String,
}

#[derive(Debug, Serialize)]
struct TraefikService {
    #[serde(rename = "loadBalancer")]
    load_balancer: TraefikLoadBalancer,
}

#[derive(Debug, Serialize)]
struct TraefikLoadBalancer {
    servers: Vec<TraefikServer>,
}

#[derive(Debug, Serialize)]
struct TraefikServer {
    url: String,
}

fn ga4gh_path_prefix(id: ServiceId) -> &'static str {
    // Must match `ferrum-gateway` route prefixes (SynapticFour/Ferrum `nest(...)` paths).
    match id {
        ServiceId::Drs => "/ga4gh/drs/v1",
        ServiceId::Htsget => "/ga4gh/htsget/v1",
        ServiceId::Wes => "/ga4gh/wes/v1",
        ServiceId::Tes => "/ga4gh/tes/v1",
        ServiceId::Beacon => "/ga4gh/beacon/v2",
        ServiceId::Trs => "/ga4gh/trs/v2",
        ServiceId::Auth => "/passports/v1",
    }
}

fn normalize_base_url(url: &impl std::fmt::Display) -> String {
    let s = url.to_string();
    if s.ends_with('/') {
        s
    } else {
        format!("{s}/")
    }
}

fn traefik_dynamic_config_for_externals(
    registry: &ServiceRegistry,
) -> Option<TraefikDynamicConfig> {
    let mut routers: BTreeMap<String, TraefikRouter> = BTreeMap::new();
    let mut services: BTreeMap<String, TraefikService> = BTreeMap::new();

    for e in &registry.entries {
        let Some(external_base) = &e.external_base else {
            continue;
        };

        let router_name = format!("{:?}", e.id).to_lowercase();
        let service_name = format!("{router_name}-external");
        let prefix = ga4gh_path_prefix(e.id);
        routers.insert(
            router_name,
            TraefikRouter {
                rule: format!("PathPrefix(`{prefix}`)"),
                service: service_name.clone(),
            },
        );
        services.insert(
            service_name,
            TraefikService {
                load_balancer: TraefikLoadBalancer {
                    servers: vec![TraefikServer {
                        url: normalize_base_url(external_base),
                    }],
                },
            },
        );
    }

    if routers.is_empty() {
        None
    } else {
        Some(TraefikDynamicConfig {
            http: TraefikHttp { routers, services },
        })
    }
}

/// Generate a Traefik dynamic configuration file for bring-your-own external services.
///
/// The output is meant to be loaded by Traefik via `--providers.file.filename=...` (or
/// an equivalent config mount).
pub fn write_traefik_dynamic_proxy_next_to_compose(
    cfg: &LabKitConfig,
    compose_output: &Path,
) -> Result<(), DeployError> {
    let registry = ServiceRegistry::from_config(cfg);
    let Some(doc) = traefik_dynamic_config_for_externals(&registry) else {
        return Ok(());
    };

    let out_path: PathBuf = compose_output
        .parent()
        .unwrap_or(Path::new("."))
        .join("proxy-traefik-dynamic.yaml");
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(out_path, serde_yaml::to_string(&doc)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lab_kit_core::{
        AuthProviderKind, AuthSection, DrsServiceConfig, ExternalSection, LabKitConfig, LabSection,
        ServicesSection,
    };

    fn base_cfg() -> LabKitConfig {
        LabKitConfig {
            schema_version: 1,
            lab: LabSection {
                name: "lab".into(),
                contact: None,
                environment: "demo".into(),
            },
            auth: AuthSection {
                provider: AuthProviderKind::None,
                ls_login: None,
                keycloak: None,
                ldap: None,
            },
            services: ServicesSection {
                drs: Some(DrsServiceConfig {
                    external_url: Some(url::Url::parse("https://drs.example").unwrap()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            external: ExternalSection::default(),
            ferrum: Default::default(),
        }
    }

    #[test]
    fn traefik_config_contains_drs() {
        let cfg = base_cfg();
        let registry = ServiceRegistry::from_config(&cfg);
        let Some(doc) = traefik_dynamic_config_for_externals(&registry) else {
            panic!("expected externals");
        };
        let yaml = serde_yaml::to_string(&doc).unwrap();
        assert!(yaml.contains("drs:"));
        assert!(yaml.contains("/ga4gh/drs/v1"));
        assert!(yaml.contains("https://drs.example/"));
    }

    #[test]
    fn ga4gh_prefixes_are_reasonable() {
        assert_eq!(ga4gh_path_prefix(ServiceId::Drs), "/ga4gh/drs/v1");
        assert_eq!(ga4gh_path_prefix(ServiceId::Beacon), "/ga4gh/beacon/v2");
        assert_eq!(ga4gh_path_prefix(ServiceId::Trs), "/ga4gh/trs/v2");
        assert_eq!(ga4gh_path_prefix(ServiceId::Htsget), "/ga4gh/htsget/v1");
        assert_eq!(ga4gh_path_prefix(ServiceId::Auth), "/passports/v1");
    }

    #[test]
    fn no_externals_yields_none() {
        let mut cfg = base_cfg();
        cfg.services.drs.as_mut().unwrap().external_url = None;
        let registry = ServiceRegistry::from_config(&cfg);
        assert!(traefik_dynamic_config_for_externals(&registry).is_none());
    }
}
