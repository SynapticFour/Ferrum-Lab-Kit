use std::fs;
use std::path::Path;

use lab_kit_core::{is_solum_enabled, LabKitConfig, ServiceId, ServiceRegistry};
use serde::Serialize;

use crate::DeployError;

/// Write Helm-style `values` for the monolith gateway + selective ENABLE_* flags.
pub fn generate_helm_values(cfg: &LabKitConfig, output_path: &Path) -> Result<(), DeployError> {
    let registry = ServiceRegistry::from_config(cfg);
    let enable = EnableFlags {
        drs: registry.is_deployed(ServiceId::Drs),
        htsget: registry.is_deployed(ServiceId::Htsget),
        wes: registry.is_deployed(ServiceId::Wes),
        tes: registry.is_deployed(ServiceId::Tes),
        beacon: registry.is_deployed(ServiceId::Beacon),
        trs: registry.is_deployed(ServiceId::Trs),
    };

    let solum_enabled = is_solum_enabled(cfg);
    let root = HelmRoot {
        global: GlobalVals {
            image_registry: "ghcr.io/synapticfour".into(),
        },
        lab: HelmLab {
            name: cfg.lab.name.clone(),
            environment: cfg.lab.environment.clone(),
        },
        gateway: GatewayVals {
            enabled: enable.any(),
            image: crate::default_ferrum_image_for(crate::FerrumImageVariant::from_config(cfg))
                .into(),
            port: 8080,
            enable,
        },
        solum: SolumVals {
            enabled: solum_enabled,
            image: "synapticfour/solum-sidecar:lab-kit".into(),
            port: cfg.solum.as_ref().map(|s| s.port).unwrap_or(8787),
        },
        auth: AuthVals {
            ls_login: LsLoginVals {
                issuer: cfg
                    .auth
                    .ls_login
                    .as_ref()
                    .map(|l| l.issuer.clone())
                    .unwrap_or_else(|| "https://login.elixir-czech.org/oidc/".to_string()),
            },
        },
        services: LegacyServicesVals {
            note: "Prefer gateway.enable*; per-service images are unpublished placeholders".into(),
        },
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, serde_yaml::to_string(&root)?)?;
    Ok(())
}

#[derive(Serialize, Default)]
struct EnableFlags {
    drs: bool,
    htsget: bool,
    wes: bool,
    tes: bool,
    beacon: bool,
    trs: bool,
}

impl EnableFlags {
    fn any(&self) -> bool {
        self.drs || self.htsget || self.wes || self.tes || self.beacon || self.trs
    }
}

#[derive(Serialize)]
struct HelmRoot {
    global: GlobalVals,
    lab: HelmLab,
    gateway: GatewayVals,
    solum: SolumVals,
    auth: AuthVals,
    services: LegacyServicesVals,
}

#[derive(Serialize)]
struct GlobalVals {
    #[serde(rename = "imageRegistry")]
    image_registry: String,
}

#[derive(Serialize)]
struct HelmLab {
    name: String,
    environment: String,
}

#[derive(Serialize)]
struct GatewayVals {
    enabled: bool,
    image: String,
    port: u16,
    enable: EnableFlags,
}

#[derive(Serialize)]
struct SolumVals {
    enabled: bool,
    image: String,
    port: u16,
}

#[derive(Serialize)]
struct LegacyServicesVals {
    note: String,
}

#[derive(Serialize)]
struct AuthVals {
    #[serde(rename = "lsLogin")]
    ls_login: LsLoginVals,
}

#[derive(Serialize)]
struct LsLoginVals {
    issuer: String,
}

#[cfg(test)]
mod tests {
    use lab_kit_core::parse_config;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn helm_values_enable_beacon_only() {
        let raw = r#"
schema_version = 1

[lab]
name = "Helm Lab"
environment = "demo"

[auth]
provider = "local"

[services.beacon]
dataset_id = "ds1"
"#;
        let cfg = parse_config(raw).unwrap();
        let dir = tempdir().unwrap();
        let out = dir.path().join("values.yaml");
        generate_helm_values(&cfg, &out).unwrap();
        let yaml = fs::read_to_string(&out).unwrap();
        assert!(yaml.contains("name: Helm Lab"));
        assert!(yaml.contains("gateway:"));
        assert!(yaml.contains("ghcr.io/synapticfour/ferrum"));
        assert!(
            yaml.contains("-edge"),
            "beacon-only helm values should pin the edge image"
        );
        assert!(yaml.contains("beacon: true"));
        assert!(yaml.contains("drs: false"));
    }
}
