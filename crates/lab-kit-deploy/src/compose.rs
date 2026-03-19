use std::fs;
use std::path::{Path, PathBuf};

use lab_kit_core::{LabKitConfig, ServiceId, ServiceRegistry};
use serde_yaml::Value;

use crate::routing::write_external_upstreams_next_to_compose;
use crate::DeployError;

fn fragment_path(fragments_dir: &Path, name: &str) -> PathBuf {
    fragments_dir.join(name)
}

/// Merge `docker-compose.base.yml` with optional per-service fragments into `output_path`.
pub fn generate_compose_file(
    cfg: &LabKitConfig,
    fragments_dir: &Path,
    output_path: &Path,
) -> Result<(), DeployError> {
    let registry = ServiceRegistry::from_config(cfg);
    let base_raw = fs::read_to_string(fragment_path(fragments_dir, "docker-compose.base.yml"))?;
    let mut merged: Value = serde_yaml::from_str(&base_raw)?;

    let mut add = |file: &str| -> Result<(), DeployError> {
        let p = fragment_path(fragments_dir, file);
        if !p.exists() {
            return Ok(());
        }
        let raw = fs::read_to_string(&p)?;
        let patch: Value = serde_yaml::from_str(&raw)?;
        merge_yaml(&mut merged, patch);
        Ok(())
    };

    for e in &registry.entries {
        if !e.deploy {
            continue;
        }
        match e.id {
            ServiceId::Drs => add("docker-compose.drs.yml")?,
            ServiceId::Htsget => add("docker-compose.htsget.yml")?,
            ServiceId::Wes => add("docker-compose.wes.yml")?,
            ServiceId::Tes => add("docker-compose.tes.yml")?,
            ServiceId::Beacon => add("docker-compose.beacon.yml")?,
            ServiceId::Trs => add("docker-compose.trs.yml")?,
            ServiceId::Auth => add("docker-compose.auth.yml")?,
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let out = serde_yaml::to_string(&merged)?;
    fs::write(output_path, out)?;
    write_external_upstreams_next_to_compose(cfg, output_path)?;
    Ok(())
}

fn merge_yaml(base: &mut Value, patch: Value) {
    match (base, patch) {
        (Value::Mapping(bm), Value::Mapping(pm)) => {
            for (k, v) in pm {
                if let Some(existing) = bm.get_mut(&k) {
                    merge_yaml(existing, v);
                } else {
                    bm.insert(k, v);
                }
            }
        }
        (b, p) => *b = p,
    }
}
