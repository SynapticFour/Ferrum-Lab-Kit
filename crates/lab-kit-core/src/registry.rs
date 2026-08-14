//! Which GA4GH surfaces are active and how to reach them (deployed vs external).

use std::fmt;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::config::{BeaconServiceConfig, LabKitConfig, SlurmNestedConfig};

/// Identifiers for GA4GH-style services wired by Lab Kit / Ferrum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceId {
    Drs,
    Htsget,
    Wes,
    Tes,
    Beacon,
    Trs,
    Auth,
}

impl ServiceId {
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceId::Drs => "drs",
            ServiceId::Htsget => "htsget",
            ServiceId::Wes => "wes",
            ServiceId::Tes => "tes",
            ServiceId::Beacon => "beacon",
            ServiceId::Trs => "trs",
            ServiceId::Auth => "auth",
        }
    }

    /// Ferrum monolith enable flag, if this surface has one.
    pub fn enable_env(self) -> Option<&'static str> {
        match self {
            ServiceId::Drs => Some("FERRUM_SERVICES__ENABLE_DRS"),
            ServiceId::Htsget => Some("FERRUM_SERVICES__ENABLE_HTSGET"),
            ServiceId::Wes => Some("FERRUM_SERVICES__ENABLE_WES"),
            ServiceId::Tes => Some("FERRUM_SERVICES__ENABLE_TES"),
            ServiceId::Beacon => Some("FERRUM_SERVICES__ENABLE_BEACON"),
            ServiceId::Trs => Some("FERRUM_SERVICES__ENABLE_TRS"),
            ServiceId::Auth => None,
        }
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistryEntry {
    pub id: ServiceId,
    /// When `false`, traffic should be routed to `external_base` or global `[external]` URLs.
    pub deploy: bool,
    #[serde(default)]
    pub external_base: Option<Url>,
    /// Used by [`crate::health::HealthAggregator`] and HelixTest pre-flight.
    #[serde(default)]
    pub health_url: Option<Url>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceRegistry {
    pub entries: Vec<ServiceRegistryEntry>,
}

impl ServiceRegistry {
    pub fn from_config(cfg: &LabKitConfig) -> Self {
        let mut entries = Vec::new();
        let s = &cfg.services;
        let health = gateway_health(cfg);

        push_service(
            &mut entries,
            ServiceId::Drs,
            s.drs
                .as_ref()
                .map(|c| (c.external_url.clone(), health.clone())),
        );
        push_service(
            &mut entries,
            ServiceId::Htsget,
            s.htsget
                .as_ref()
                .map(|c| (c.external_url.clone(), health.clone())),
        );
        if s.htsget.is_none() {
            if let Some(url) = cfg.external.htsget_url.clone() {
                entries.push(ServiceRegistryEntry {
                    id: ServiceId::Htsget,
                    deploy: false,
                    health_url: Some(url.clone()),
                    external_base: Some(url),
                });
            }
        }
        push_service(
            &mut entries,
            ServiceId::Wes,
            s.wes
                .as_ref()
                .map(|c| (c.external_url.clone(), health.clone())),
        );
        push_service(
            &mut entries,
            ServiceId::Tes,
            s.tes
                .as_ref()
                .map(|c| (c.external_url.clone(), health.clone())),
        );
        push_service(
            &mut entries,
            ServiceId::Beacon,
            s.beacon
                .as_ref()
                .map(|c: &BeaconServiceConfig| (c.external_url.clone(), health.clone())),
        );
        push_service(
            &mut entries,
            ServiceId::Trs,
            s.trs
                .as_ref()
                .map(|c| (c.external_url.clone(), health.clone())),
        );

        // Identity providers are configured on the monolith gateway. `deploy: true`
        // only matters for `--legacy-per-service` (auth-proxy fragment).
        if matches!(
            cfg.auth.provider,
            crate::config::AuthProvider::LsLogin | crate::config::AuthProvider::Keycloak
        ) {
            entries.push(ServiceRegistryEntry {
                id: ServiceId::Auth,
                deploy: true,
                external_base: None,
                health_url: health,
            });
        }

        Self { entries }
    }

    pub fn enabled_ids(&self) -> impl Iterator<Item = ServiceId> + '_ {
        self.entries.iter().map(|e| e.id)
    }

    pub fn is_deployed(&self, id: ServiceId) -> bool {
        self.entries.iter().any(|e| e.id == id && e.deploy)
    }
}

/// TES SLURM block, falling back to WES when TES omits `[services.tes.slurm]`.
pub fn tes_slurm_config(cfg: &LabKitConfig) -> Option<&SlurmNestedConfig> {
    cfg.services
        .tes
        .as_ref()
        .and_then(|t| t.slurm.as_ref())
        .or_else(|| cfg.services.wes.as_ref().and_then(|w| w.slurm.as_ref()))
}

fn push_service(
    entries: &mut Vec<ServiceRegistryEntry>,
    id: ServiceId,
    opt: Option<(Option<Url>, Option<Url>)>,
) {
    let Some((external, health)) = opt else {
        return;
    };
    let deploy = external.is_none();
    let health_url = if deploy { health } else { external.clone() };
    entries.push(ServiceRegistryEntry {
        id,
        deploy,
        external_base: external,
        health_url,
    });
}

fn join_health(base: &Url) -> Option<Url> {
    let mut u = base.clone();
    if u.path().ends_with("/health") {
        return Some(u);
    }
    let path = u.path().trim_end_matches('/');
    u.set_path(&format!("{path}/health"));
    Some(u)
}

/// Default health checks for the monolith gateway.
pub fn gateway_health(cfg: &LabKitConfig) -> Option<Url> {
    if let Some(base) = cfg.ferrum.gateway_url.as_ref() {
        return join_health(base);
    }
    if let Ok(u) = std::env::var("FERRUM_GATEWAY_URL") {
        if !u.trim().is_empty() {
            if let Ok(parsed) = Url::parse(u.trim()) {
                return join_health(&parsed);
            }
        }
    }
    let port = std::env::var("FERRUM_PORT")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "8080".into());
    Url::parse(&format!("http://127.0.0.1:{port}/health")).ok()
}
