use std::fs;
use std::path::{Path, PathBuf};

use lab_kit_core::{
    is_co_deploy, is_solum_enabled, Ga4ghInfraMode, LabKitConfig, ServiceId, ServiceRegistry,
};
use serde_yaml::{Mapping, Value};

use crate::routing::{
    write_external_upstreams_next_to_compose, write_traefik_dynamic_proxy_next_to_compose,
};
use crate::DeployError;

/// Options for [`generate_compose_file`].
#[derive(Debug, Clone, Default)]
pub struct ComposeOptions {
    /// Merge ga4gh-infra even when `[ga4gh_infra]` is unset.
    pub with_ga4gh_infra: bool,
    /// Merge Solum sidecar even when `[solum]` is unset.
    pub with_solum: bool,
    /// Use unpublished per-service image fragments instead of the monolith gateway.
    pub legacy_per_service: bool,
}

fn fragment_path(fragments_dir: &Path, name: &str) -> PathBuf {
    fragments_dir.join(name)
}

fn merge_fragment(merged: &mut Value, fragments_dir: &Path, file: &str) -> Result<(), DeployError> {
    let p = fragment_path(fragments_dir, file);
    if !p.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&p)?;
    let patch: Value = serde_yaml::from_str(&raw)?;
    merge_yaml(merged, patch);
    Ok(())
}

/// Merge `docker-compose.base.yml` with gateway (or legacy fragments) into `output_path`.
pub fn generate_compose_file(
    cfg: &LabKitConfig,
    fragments_dir: &Path,
    output_path: &Path,
    options: &ComposeOptions,
) -> Result<(), DeployError> {
    let registry = ServiceRegistry::from_config(cfg);
    let base_raw = fs::read_to_string(fragment_path(fragments_dir, "docker-compose.base.yml"))?;
    let mut merged: Value = serde_yaml::from_str(&base_raw)?;

    let any_ga4gh_deploy = registry.entries.iter().any(|e| {
        e.deploy
            && matches!(
                e.id,
                ServiceId::Drs
                    | ServiceId::Htsget
                    | ServiceId::Wes
                    | ServiceId::Tes
                    | ServiceId::Beacon
                    | ServiceId::Trs
            )
    });

    if options.legacy_per_service {
        for e in &registry.entries {
            if !e.deploy {
                continue;
            }
            match e.id {
                ServiceId::Drs => {
                    merge_fragment(&mut merged, fragments_dir, "docker-compose.drs.yml")?
                }
                ServiceId::Htsget => {
                    merge_fragment(&mut merged, fragments_dir, "docker-compose.htsget.yml")?
                }
                ServiceId::Wes => {
                    merge_fragment(&mut merged, fragments_dir, "docker-compose.wes.yml")?
                }
                ServiceId::Tes => {
                    merge_fragment(&mut merged, fragments_dir, "docker-compose.tes.yml")?
                }
                ServiceId::Beacon => {
                    merge_fragment(&mut merged, fragments_dir, "docker-compose.beacon.yml")?
                }
                ServiceId::Trs => {
                    merge_fragment(&mut merged, fragments_dir, "docker-compose.trs.yml")?
                }
                ServiceId::Auth => {
                    merge_fragment(&mut merged, fragments_dir, "docker-compose.auth.yml")?
                }
            }
        }
    } else if any_ga4gh_deploy {
        merge_fragment(&mut merged, fragments_dir, "docker-compose.gateway.yml")?;
        apply_enable_flags(&mut merged, &registry)?;
    }

    if lab_kit_core::is_field_edge(cfg) {
        merge_fragment(&mut merged, fragments_dir, "edge.yml")?;
    }

    let ga4gh = cfg.ga4gh_infra.as_ref();
    let force_infra = options.with_ga4gh_infra;
    let co_deploy = force_infra || is_co_deploy(cfg);
    let external_infra = ga4gh.is_some_and(|g| g.enabled && g.mode == Ga4ghInfraMode::External);

    if co_deploy {
        merge_fragment(&mut merged, fragments_dir, "infra.yml")?;
        merge_fragment(&mut merged, fragments_dir, "co-deploy.yml")?;
        if let Some(g) = ga4gh {
            apply_broker_port(&mut merged, g.broker_port)?;
        }
    } else if external_infra {
        merge_fragment(&mut merged, fragments_dir, "co-deploy-external.yml")?;
        apply_external_infra_urls(&mut merged, ga4gh.expect("checked"))?;
    }

    let solum = options.with_solum || is_solum_enabled(cfg);
    if solum {
        merge_fragment(&mut merged, fragments_dir, "solum.yml")?;
        apply_solum_defaults(&mut merged, cfg)?;
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let out = serde_yaml::to_string(&merged)?;
    fs::write(output_path, out)?;
    write_external_upstreams_next_to_compose(cfg, output_path)?;
    write_traefik_dynamic_proxy_next_to_compose(cfg, output_path)?;
    Ok(())
}

fn gateway_env_map(merged: &mut Value) -> Result<&mut Mapping, DeployError> {
    let services = merged
        .as_mapping_mut()
        .ok_or_else(|| DeployError::Msg("compose root must be a mapping".into()))?
        .entry(Value::String("services".into()))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    let services_map = services
        .as_mapping_mut()
        .ok_or_else(|| DeployError::Msg("services must be a mapping".into()))?;
    let gateway = services_map
        .entry(Value::String("ferrum-gateway".into()))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    let gateway_map = gateway
        .as_mapping_mut()
        .ok_or_else(|| DeployError::Msg("ferrum-gateway must be a mapping".into()))?;
    let env = gateway_map
        .entry(Value::String("environment".into()))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    env.as_mapping_mut()
        .ok_or_else(|| DeployError::Msg("ferrum-gateway.environment must be a mapping".into()))
}

fn apply_enable_flags(merged: &mut Value, registry: &ServiceRegistry) -> Result<(), DeployError> {
    let flag = |id: ServiceId| -> &'static str {
        if registry.entries.iter().any(|e| e.id == id && e.deploy) {
            "true"
        } else {
            "false"
        }
    };
    let patch_raw = format!(
        r#"
services:
  ferrum-gateway:
    environment:
      FERRUM_SERVICES__ENABLE_DRS: "{drs}"
      FERRUM_SERVICES__ENABLE_HTSGET: "{htsget}"
      FERRUM_SERVICES__ENABLE_WES: "{wes}"
      FERRUM_SERVICES__ENABLE_TES: "{tes}"
      FERRUM_SERVICES__ENABLE_BEACON: "{beacon}"
      FERRUM_SERVICES__ENABLE_TRS: "{trs}"
"#,
        drs = flag(ServiceId::Drs),
        htsget = flag(ServiceId::Htsget),
        wes = flag(ServiceId::Wes),
        tes = flag(ServiceId::Tes),
        beacon = flag(ServiceId::Beacon),
        trs = flag(ServiceId::Trs),
    );
    let patch: Value = serde_yaml::from_str(&patch_raw)?;
    merge_yaml(merged, patch);
    Ok(())
}

fn apply_broker_port(merged: &mut Value, broker_port: u16) -> Result<(), DeployError> {
    if broker_port == 8180 {
        return Ok(());
    }
    let Some(root) = merged.as_mapping_mut() else {
        return Ok(());
    };
    let Some(services) = root
        .get_mut(Value::String("services".into()))
        .and_then(|v| v.as_mapping_mut())
    else {
        return Ok(());
    };
    let Some(broker) = services
        .get_mut(Value::String("aai-broker".into()))
        .and_then(|v| v.as_mapping_mut())
    else {
        return Ok(());
    };
    let ports = Value::Sequence(vec![Value::String(format!("{broker_port}:8080"))]);
    broker.insert(Value::String("ports".into()), ports);
    Ok(())
}

fn apply_external_infra_urls(
    merged: &mut Value,
    ga4gh: &lab_kit_core::Ga4ghInfraSection,
) -> Result<(), DeployError> {
    let env = gateway_env_map(merged)?;
    let broker = ga4gh.broker_port;
    let issuer = format!("http://127.0.0.1:{broker}");
    let jwks = format!("{issuer}/jwks.json");
    let registry_url = ga4gh
        .service_registry_url
        .clone()
        .unwrap_or_else(|| "http://127.0.0.1:8183".into());
    env.insert(
        Value::String("FERRUM_AUTH__JWKS_URL".into()),
        Value::String(jwks),
    );
    env.insert(
        Value::String("FERRUM_AUTH__ISSUER".into()),
        Value::String(issuer),
    );
    env.insert(
        Value::String("FERRUM_DISCOVERY__SERVICE_REGISTRY_URL".into()),
        Value::String(registry_url),
    );
    env.insert(
        Value::String("FERRUM_DISCOVERY__REGISTRATION_API_KEY_ENV".into()),
        Value::String(ga4gh.registration_api_key_env.clone()),
    );
    Ok(())
}

fn apply_solum_defaults(merged: &mut Value, cfg: &LabKitConfig) -> Result<(), DeployError> {
    let env = gateway_env_map(merged)?;
    if let Some(solum) = cfg.solum.as_ref() {
        if let Some(ref subject) = solum.default_subject {
            env.insert(
                Value::String("FERRUM_SOLUM__DEFAULT_SUBJECT".into()),
                Value::String(subject.clone()),
            );
        }
        if let Some(ref purpose) = solum.default_purpose {
            env.insert(
                Value::String("FERRUM_SOLUM__DEFAULT_PURPOSE".into()),
                Value::String(purpose.clone()),
            );
        }
        env.insert(
            Value::String("FERRUM_SOLUM__TIMEOUT_SECS".into()),
            Value::String(solum.timeout_secs.to_string()),
        );
    }

    // Merge depends_on without clobbering ga4gh-infra broker deps.
    let patch: Value = serde_yaml::from_str(
        r#"
services:
  ferrum-gateway:
    depends_on:
      solum-sidecar:
        condition: service_started
"#,
    )?;
    merge_yaml(merged, patch);
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

#[cfg(test)]
mod tests {
    use lab_kit_core::parse_config;

    use super::*;

    fn fragments() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deploy/docker-compose")
    }

    #[test]
    fn field_edge_compose_includes_gateway_overlay() {
        let raw = include_str!("../../../config/profiles/field-edge.toml");
        let cfg = parse_config(raw).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("docker-compose.yml");
        generate_compose_file(&cfg, &fragments(), &out, &ComposeOptions::default()).unwrap();
        let merged = std::fs::read_to_string(&out).unwrap();
        assert!(merged.contains("ferrum-gateway"));
        assert!(merged.contains("ghcr.io/synapticfour/ferrum"));
        assert!(merged.contains("FERRUM_SERVICES__ENABLE_BEACON"));
        assert!(merged.contains("FERRUM_AFRICA__OFFLINE_FIRST"));
        assert!(!merged.contains("synapticfour/ferrum-beacon"));
        serde_yaml::from_str::<serde_yaml::Value>(&merged).expect("valid YAML");
    }

    #[test]
    fn field_edge_infra_profile_merges_ga4gh_stack() {
        let raw = include_str!("../../../config/profiles/field-edge+infra.toml");
        let cfg = parse_config(raw).unwrap();
        assert!(lab_kit_core::is_co_deploy(&cfg));
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("docker-compose.yml");
        generate_compose_file(&cfg, &fragments(), &out, &ComposeOptions::default()).unwrap();
        let merged = std::fs::read_to_string(&out).unwrap();
        assert!(merged.contains("aai-broker"));
        assert!(merged.contains("mock-idp"));
        assert!(merged.contains("FERRUM_AUTH__JWKS_URL"));
        assert!(merged.contains("FERRUM_SERVICES__ENABLE_PASSPORTS"));
        assert!(merged.contains("FERRUM_DISCOVERY__ENABLED"));
        assert!(merged.contains("8180:8080"));
        assert!(merged.contains("ghcr.io/synapticfour/ferrum"));
        serde_yaml::from_str::<serde_yaml::Value>(&merged).expect("valid YAML");
    }

    #[test]
    fn cli_with_ga4gh_infra_flag_merges_infra_without_config() {
        let raw = include_str!("../../../config/profiles/field-edge.toml");
        let cfg = parse_config(raw).unwrap();
        assert!(!lab_kit_core::is_co_deploy(&cfg));
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("docker-compose.yml");
        generate_compose_file(
            &cfg,
            &fragments(),
            &out,
            &ComposeOptions {
                with_ga4gh_infra: true,
                ..Default::default()
            },
        )
        .unwrap();
        let merged = std::fs::read_to_string(&out).unwrap();
        assert!(merged.contains("aai-broker"));
        assert!(merged.contains("FERRUM_AUTH__JWKS_URL"));
        serde_yaml::from_str::<serde_yaml::Value>(&merged).expect("valid YAML");
    }

    #[test]
    fn beacon_only_compose_uses_monolith_with_beacon_enabled() {
        let raw = include_str!("../../../config/profiles/beacon-only.toml");
        let cfg = parse_config(raw).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("docker-compose.yml");
        generate_compose_file(&cfg, &fragments(), &out, &ComposeOptions::default()).unwrap();
        let merged = std::fs::read_to_string(&out).unwrap();
        assert!(merged.contains("ferrum-gateway:"));
        assert!(merged.contains("ghcr.io/synapticfour/ferrum"));
        assert!(
            merged.contains("FERRUM_SERVICES__ENABLE_BEACON"),
            "expected ENABLE_BEACON in compose"
        );
        // serde_yaml may emit true/false quoted or bare; require beacon true and drs false.
        let beacon_line = merged
            .lines()
            .find(|l| l.contains("FERRUM_SERVICES__ENABLE_BEACON"))
            .unwrap_or("");
        assert!(
            beacon_line.contains("true"),
            "beacon should be enabled, got: {beacon_line}"
        );
        assert!(!merged.contains("aai-broker"));
        assert!(!merged.contains("synapticfour/ferrum-beacon"));
        serde_yaml::from_str::<serde_yaml::Value>(&merged).expect("valid YAML");
    }

    #[test]
    fn disabled_services_are_omitted_from_enable_flags() {
        let raw = r#"
schema_version = 1

[lab]
name = "Beacon Only"
environment = "demo"

[auth]
provider = "local"

[services.beacon]
dataset_id = "ds1"
"#;
        let cfg = parse_config(raw).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("docker-compose.yml");
        generate_compose_file(&cfg, &fragments(), &out, &ComposeOptions::default()).unwrap();
        let merged = std::fs::read_to_string(&out).unwrap();
        assert!(merged.contains("ENABLE_BEACON"));
        let drs_line = merged
            .lines()
            .find(|l| l.contains("FERRUM_SERVICES__ENABLE_DRS"))
            .unwrap_or("");
        assert!(
            drs_line.contains("false"),
            "DRS should be disabled, got: {drs_line}"
        );
        assert!(!merged.contains("\n  drs:"));
        assert!(!merged.contains("\n  wes:"));
    }

    #[test]
    fn with_solum_merges_sidecar_and_ferrum_env() {
        let raw = r#"
schema_version = 1

[lab]
name = "With Solum"
environment = "demo"

[auth]
provider = "local"

[services.beacon]
dataset_id = "ds1"

[services.drs]

[solum]
enabled = true
default_subject = "patient-1"
default_purpose = "research"
"#;
        let cfg = parse_config(raw).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("docker-compose.yml");
        generate_compose_file(&cfg, &fragments(), &out, &ComposeOptions::default()).unwrap();
        let merged = std::fs::read_to_string(&out).unwrap();
        assert!(merged.contains("solum-sidecar"));
        assert!(merged.contains("FERRUM_SOLUM__BASE_URL"));
        assert!(merged.contains("FERRUM_SOLUM__DEFAULT_SUBJECT"));
        assert!(merged.contains("patient-1"));
        serde_yaml::from_str::<serde_yaml::Value>(&merged).expect("valid YAML");
    }

    #[test]
    fn legacy_per_service_still_emits_placeholder_beacon() {
        let raw = include_str!("../../../config/profiles/beacon-only.toml");
        let cfg = parse_config(raw).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("docker-compose.yml");
        generate_compose_file(
            &cfg,
            &fragments(),
            &out,
            &ComposeOptions {
                legacy_per_service: true,
                ..Default::default()
            },
        )
        .unwrap();
        let merged = std::fs::read_to_string(&out).unwrap();
        assert!(merged.contains("beacon:"));
        assert!(merged.contains("synapticfour/ferrum-beacon"));
    }
}
