// SPDX-License-Identifier: BUSL-1.1
use std::fs;
use std::path::Path;

use lab_kit_core::{is_solum_enabled, tes_slurm_config, LabKitConfig, ServiceId, ServiceRegistry};
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
            adapters: adapter_runtime(cfg),
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
    adapters: AdapterRuntime,
}

/// Maps `lab-kit-adapters` config (POSIX/S3/SLURM) onto Ferrum env consumed by the gateway chart.
#[derive(Serialize, Default)]
struct AdapterRuntime {
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_base_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    s3_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    s3_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    s3_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wes_backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tes_backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wes_slurm_partition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tes_slurm_partition: Option<String>,
}

fn adapter_runtime(cfg: &LabKitConfig) -> AdapterRuntime {
    let mut a = AdapterRuntime::default();
    if let Some(drs) = cfg.services.drs.as_ref() {
        if let Some(s3) = &drs.s3 {
            a.storage_backend = Some("s3".into());
            a.s3_endpoint = Some(s3.endpoint.as_str().to_string());
            a.s3_bucket = Some(s3.bucket.clone());
            a.s3_region = s3.region.clone();
        } else if let Some(posix) = &drs.posix {
            a.storage_backend = Some("local".into());
            a.storage_base_path = Some(posix.root.clone());
        }
    }
    if a.storage_backend.is_none() {
        if let Some(backend) = cfg.backend.as_ref() {
            a.storage_backend = Some("local".into());
            a.storage_base_path = Some(backend.objects_path.clone());
        }
    }
    if let Some(wes) = cfg.services.wes.as_ref() {
        if wes
            .compute_backend
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("slurm"))
        {
            a.wes_backend = Some("slurm".into());
        }
        if let Some(p) = wes.slurm.as_ref().and_then(|s| s.partition.as_ref()) {
            a.wes_slurm_partition = Some(p.clone());
        }
    }
    if let Some(tes) = cfg.services.tes.as_ref() {
        if tes
            .compute_backend
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("slurm"))
        {
            a.tes_backend = Some("slurm".into());
        }
        if let Some(p) = tes_slurm_config(cfg).and_then(|s| s.partition.as_ref()) {
            a.tes_slurm_partition = Some(p.clone());
        }
    }
    a
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

    #[test]
    fn helm_values_wire_posix_and_slurm_adapters() {
        let raw = r#"
schema_version = 1

[lab]
name = "Adapter Lab"
environment = "demo"

[auth]
provider = "local"

[services.drs]
storage_backend = "posix"

[services.drs.posix]
root = "/data/objects"

[services.wes]
compute_backend = "slurm"

[services.wes.slurm]
partition = "batch"

[services.tes]
compute_backend = "slurm"
"#;
        let cfg = parse_config(raw).unwrap();
        let dir = tempdir().unwrap();
        let out = dir.path().join("values.yaml");
        generate_helm_values(&cfg, &out).unwrap();
        let yaml = fs::read_to_string(&out).unwrap();
        assert!(yaml.contains("storage_backend: local"));
        assert!(yaml.contains("storage_base_path: /data/objects"));
        assert!(yaml.contains("wes_backend: slurm"));
        assert!(yaml.contains("tes_backend: slurm"));
        assert!(yaml.contains("wes_slurm_partition: batch"));
    }
}
