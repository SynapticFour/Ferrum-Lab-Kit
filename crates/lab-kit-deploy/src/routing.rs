use std::fs;
use std::path::{Path, PathBuf};

use lab_kit_core::{LabKitConfig, ServiceRegistry};
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
