//! Which GA4GH surfaces are active and how to reach them (deployed vs external).

use serde::{Deserialize, Serialize};
use url::Url;

use crate::config::{BeaconServiceConfig, LabKitConfig};

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

        push_service(
            &mut entries,
            ServiceId::Drs,
            s.drs
                .as_ref()
                .map(|c| (c.external_url.clone(), default_health(ServiceId::Drs))),
        );
        push_service(
            &mut entries,
            ServiceId::Htsget,
            s.htsget
                .as_ref()
                .map(|c| (c.external_url.clone(), default_health(ServiceId::Htsget))),
        );
        push_service(
            &mut entries,
            ServiceId::Wes,
            s.wes
                .as_ref()
                .map(|c| (c.external_url.clone(), default_health(ServiceId::Wes))),
        );
        push_service(
            &mut entries,
            ServiceId::Tes,
            s.tes.as_ref().map(|c| {
                let slurm = c
                    .slurm
                    .clone()
                    .or_else(|| s.wes.as_ref().and_then(|w| w.slurm.clone()));
                let _ = slurm; // inherit rule documented; deploy generator uses full config
                (c.external_url.clone(), default_health(ServiceId::Tes))
            }),
        );
        push_service(
            &mut entries,
            ServiceId::Beacon,
            s.beacon.as_ref().map(|c: &BeaconServiceConfig| {
                (c.external_url.clone(), default_health(ServiceId::Beacon))
            }),
        );
        push_service(
            &mut entries,
            ServiceId::Trs,
            s.trs
                .as_ref()
                .map(|c| (c.external_url.clone(), default_health(ServiceId::Trs))),
        );

        if matches!(
            cfg.auth.provider,
            crate::config::AuthProvider::LsLogin | crate::config::AuthProvider::Keycloak
        ) {
            entries.push(ServiceRegistryEntry {
                id: ServiceId::Auth,
                deploy: true,
                external_base: None,
                health_url: default_health(ServiceId::Auth),
            });
        }

        Self { entries }
    }

    pub fn enabled_ids(&self) -> impl Iterator<Item = ServiceId> + '_ {
        self.entries.iter().map(|e| e.id)
    }
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

/// Default localhost health checks for generated Compose stacks (Ferrum images expose `/health`).
fn default_health(id: ServiceId) -> Option<Url> {
    let port = match id {
        ServiceId::Drs => 8081,
        ServiceId::Htsget => 8082,
        ServiceId::Wes => 8083,
        ServiceId::Tes => 8084,
        ServiceId::Beacon => 8085,
        ServiceId::Trs => 8086,
        ServiceId::Auth => 8090,
    };
    Url::parse(&format!("http://127.0.0.1:{port}/health")).ok()
}
