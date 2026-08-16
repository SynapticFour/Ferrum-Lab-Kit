// SPDX-License-Identifier: BUSL-1.1
//! Portable Raspberry Pi field-edge install kit.

use std::fs;
use std::path::Path;

use lab_kit_core::{is_co_deploy, is_solum_enabled, LabKitConfig};

use crate::compose::{generate_compose_file, ComposeOptions};
use crate::DeployError;

/// Options for [`generate_raspberry_pi_bundle`].
#[derive(Debug, Clone)]
pub struct RaspberryPiBundleOptions {
    pub compose: ComposeOptions,
    /// Host data directory on the Pi (expanded by install script).
    pub data_dir: String,
    /// Expected RAM in GB (documented in README; used for memory hint).
    pub ram_gb: u32,
}

impl Default for RaspberryPiBundleOptions {
    fn default() -> Self {
        Self {
            compose: ComposeOptions::default(),
            data_dir: "~/.ferrum".into(),
            ram_gb: 4,
        }
    }
}

/// Write a self-contained Raspberry Pi install kit under `output_dir`.
///
/// Layout:
/// ```text
/// output_dir/
///   README.md
///   .env
///   lab-kit.toml
///   docker-compose.yml
///   install-on-pi.sh
///   deploy/docker-compose/   # fragments needed for regenerate / infra / solum
/// ```
pub fn generate_raspberry_pi_bundle(
    cfg: &LabKitConfig,
    fragments_dir: &Path,
    output_dir: &Path,
    options: &RaspberryPiBundleOptions,
) -> Result<(), DeployError> {
    fs::create_dir_all(output_dir)?;

    let kit_fragments = output_dir.join("deploy/docker-compose");
    fs::create_dir_all(&kit_fragments)?;
    copy_pi_fragments(fragments_dir, &kit_fragments, cfg, &options.compose)?;

    let toml = toml::to_string_pretty(cfg)
        .map_err(|e| DeployError::Msg(format!("serialize lab-kit.toml: {e}")))?;
    fs::write(output_dir.join("lab-kit.toml"), toml)?;

    let compose_out = output_dir.join("docker-compose.yml");
    generate_compose_file(cfg, &kit_fragments, &compose_out, &options.compose)?;

    // Rewrite fragment-relative paths so the kit runs from its own directory.
    rewrite_compose_paths_for_kit(&compose_out)?;

    fs::write(
        output_dir.join(".env"),
        pi_env_file(cfg, options, &options.compose)?,
    )?;
    fs::write(output_dir.join("README.md"), pi_readme(cfg, options))?;
    fs::write(output_dir.join("install-on-pi.sh"), pi_install_script())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(output_dir.join("install-on-pi.sh"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(output_dir.join("install-on-pi.sh"), perms)?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), DeployError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

fn copy_file(src_dir: &Path, dst_dir: &Path, name: &str) -> Result<(), DeployError> {
    let src = src_dir.join(name);
    if src.exists() {
        fs::copy(&src, dst_dir.join(name))?;
    }
    Ok(())
}

fn copy_pi_fragments(
    src: &Path,
    dst: &Path,
    cfg: &LabKitConfig,
    compose: &ComposeOptions,
) -> Result<(), DeployError> {
    // Always needed for regenerate / docs.
    for name in [
        "docker-compose.base.yml",
        "docker-compose.gateway.yml",
        "edge.yml",
    ] {
        copy_file(src, dst, name)?;
    }

    let want_infra = compose.with_ga4gh_infra || is_co_deploy(cfg);
    let want_external = cfg
        .ga4gh_infra
        .as_ref()
        .is_some_and(|g| g.enabled && matches!(g.mode, lab_kit_core::Ga4ghInfraMode::External));
    if want_infra {
        copy_file(src, dst, "infra.yml")?;
        copy_file(src, dst, "co-deploy.yml")?;
        let cfg_src = src.join("ga4gh-infra-config");
        let sec_src = src.join("ga4gh-infra-secrets");
        if cfg_src.is_dir() {
            copy_dir_recursive(&cfg_src, &dst.join("ga4gh-infra-config"))?;
        }
        if sec_src.is_dir() {
            copy_dir_recursive(&sec_src, &dst.join("ga4gh-infra-secrets"))?;
        }
        crate::generate_infra_secrets(&dst.join("ga4gh-infra-secrets"), false)?;
    }
    if want_external {
        copy_file(src, dst, "co-deploy-external.yml")?;
    }

    let want_solum = compose.with_solum || is_solum_enabled(cfg);
    if want_solum {
        copy_file(src, dst, "solum.yml")?;
        copy_file(src, dst, "Dockerfile.solum-sidecar")?;
    }

    Ok(())
}

fn rewrite_compose_paths_for_kit(compose_path: &Path) -> Result<(), DeployError> {
    let raw = fs::read_to_string(compose_path)?;
    // Generated compose may still reference repo-root defaults; pin kit-relative paths.
    let patched = raw
        .replace(
            "${GA4GH_INFRA_CONFIG_DIR:-deploy/docker-compose/ga4gh-infra-config}",
            "${GA4GH_INFRA_CONFIG_DIR:-./deploy/docker-compose/ga4gh-infra-config}",
        )
        .replace(
            "${GA4GH_INFRA_SECRETS_DIR:-deploy/docker-compose/ga4gh-infra-secrets}",
            "${GA4GH_INFRA_SECRETS_DIR:-./deploy/docker-compose/ga4gh-infra-secrets}",
        )
        .replace(
            "${SOLUM_DOCKER_CONTEXT:-deploy/docker-compose}",
            "${SOLUM_DOCKER_CONTEXT:-./deploy/docker-compose}",
        );
    fs::write(compose_path, patched)?;
    Ok(())
}

fn pi_env_file(
    cfg: &LabKitConfig,
    options: &RaspberryPiBundleOptions,
    compose: &ComposeOptions,
) -> Result<String, DeployError> {
    let max_mem = options.ram_gb.saturating_mul(768);
    let want_solum = compose.with_solum || is_solum_enabled(cfg);
    let want_infra = compose.with_ga4gh_infra || is_co_deploy(cfg);

    let mut out = String::new();
    out.push_str("# Ferrum Lab Kit — Raspberry Pi field kit\n");
    out.push_str("# Generated by: lab-kit generate raspberry-pi\n\n");
    out.push_str("# Monolith image pin (override with a digest for pilots)\n");
    out.push_str(&format!(
        "FERRUM_IMAGE={}\n",
        crate::default_ferrum_image_arm64()
    ));
    out.push_str("FERRUM_PORT=8080\n");
    out.push_str(&format!(
        "FERRUM_DATA_DIR={}\n",
        options.data_dir.replace('~', "$HOME")
    ));
    out.push_str(&format!(
        "# Suggested max_memory_mb for {ram} GB Pi: {max_mem}\n",
        ram = options.ram_gb,
        max_mem = max_mem
    ));
    out.push('\n');

    if want_infra {
        out.push_str("GA4GH_IMAGE_PREFIX=ghcr.io/synapticfour\n");
        out.push_str("GA4GH_INFRA_CONFIG_DIR=./deploy/docker-compose/ga4gh-infra-config\n");
        out.push_str("GA4GH_INFRA_SECRETS_DIR=./deploy/docker-compose/ga4gh-infra-secrets\n");
        out.push('\n');
    }

    if want_solum {
        out.push_str("# Solum Track A sidecar (consent companion). Building on Pi is slow — prefer SOLUM_IMAGE.\n");
        out.push_str("SOLUM_PORT=8787\n");
        let token = cfg
            .solum
            .as_ref()
            .and_then(|s| s.sidecar_token.clone())
            .filter(|t| !t.trim().is_empty() && t != "solum-lab-kit-local-token-change-me")
            .unwrap_or_else(|| format!("solum-{}", uuid::Uuid::new_v4()));
        out.push_str(&format!("SOLUM_SIDECAR_TOKEN={token}\n"));
        out.push_str("SOLUM_ALLOW_EPHEMERAL=1\n");
        out.push_str("SOLUM_DOCKER_CONTEXT=./deploy/docker-compose\n");
        out.push_str("# SOLUM_IMAGE=ghcr.io/example/solum-sidecar:tag\n");
        out.push('\n');
    }

    Ok(out)
}

fn pi_readme(cfg: &LabKitConfig, options: &RaspberryPiBundleOptions) -> String {
    let profile = cfg
        .meta
        .as_ref()
        .map(|m| m.profile.as_str())
        .unwrap_or("field-edge");
    let solum = is_solum_enabled(cfg) || options.compose.with_solum;
    let infra = is_co_deploy(cfg) || options.compose.with_ga4gh_infra;

    let mut companions = String::from("Ferrum monolith (Beacon + DRS)");
    if infra {
        companions.push_str(" + ga4gh-infra");
    }
    if solum {
        companions.push_str(" + Solum sidecar");
    }

    format!(
        r#"# Ferrum Lab Kit — Raspberry Pi field kit

**Profile:** `{profile}`
**Stack:** {companions}
**Target:** Raspberry Pi 5 (recommended, 4–8 GB) or Pi 4 (4 GB minimum)

This directory is a **portable install kit**. Copy it to a USB stick, clone it onto the Pi, or `scp -r` it, then run `./install-on-pi.sh` **on the Pi**.

## Hardware checklist

| Item | Guidance |
|------|----------|
| Board | **Pi 5** preferred; Pi 4 works for Beacon+DRS only |
| RAM | 4 GB minimum; **8 GB** if Solum or ga4gh-infra is included |
| Storage | Prefer **USB SSD** for `{data}` (SQLite + objects); avoid microSD for heavy ingest |
| OS | Raspberry Pi OS **64-bit** or Ubuntu 24.04 ARM64 |
| Network | Needed once to pull the pinned `FERRUM_IMAGE` from `.env` (and companions) |

## Install on the Pi

```bash
cd "$(dirname "$0")"   # this kit directory
./install-on-pi.sh
```

What it does:

1. Checks ARM64 + Docker
2. Loads `.env` (ARM64 Ferrum image)
3. `docker compose up -d --build`
4. Waits for `http://127.0.0.1:8080/health` and Beacon `/ga4gh/beacon/v2/info`

Manual equivalent:

```bash
set -a; source .env; set +a
mkdir -p "${{FERRUM_DATA_DIR:-$HOME/.ferrum}}/objects"
docker compose -f docker-compose.yml up -d --build
```

## Verify

```bash
curl -fsS http://127.0.0.1:8080/health
curl -fsS http://127.0.0.1:8080/ga4gh/beacon/v2/info
curl -fsS http://127.0.0.1:8080/ga4gh/drs/v1/service-info
{solum_verify}{infra_verify}
```

## What this kit is (and is not)

| Included | Not included |
|----------|----------------|
| Beacon v2 + DRS on Ferrum monolith | Full WES/TES/TRS (disabled by default) |
| SQLite + local filesystem | PostgreSQL / MinIO |
| Optional ga4gh-infra / Solum Track A | Solum Track B / EHRbase (hub only) |

Upstream Edge narrative: [Ferrum AFRICA-DEPLOYMENT](https://github.com/SynapticFour/Ferrum/blob/main/docs/AFRICA-DEPLOYMENT.md).
Lab Kit docs: [RASPBERRY-PI.md](https://github.com/SynapticFour/Ferrum-Lab-Kit/blob/main/docs/RASPBERRY-PI.md).

## Regenerate compose

If you edit `lab-kit.toml` on a machine with the `lab-kit` CLI:

```bash
lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose -o docker-compose.yml
```

Or rebuild the kit from a Lab Kit checkout:

```bash
lab-kit generate raspberry-pi --output ./pi-kit --profile {profile}
```
"#,
        profile = profile,
        companions = companions,
        data = options.data_dir,
        solum_verify = if solum {
            "curl -fsS -H \"X-Solum-Sidecar-Token: $SOLUM_SIDECAR_TOKEN\" http://127.0.0.1:8787/v1/health 2>/dev/null || true\n"
        } else {
            ""
        },
        infra_verify = if infra {
            "curl -fsS http://127.0.0.1:8180/service-info\n"
        } else {
            ""
        },
    )
}

fn pi_install_script() -> String {
    let pin = crate::default_ferrum_image_arm64();
    r#"#!/usr/bin/env bash
# Ferrum Lab Kit — run this ON the Raspberry Pi (inside the kit directory).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo "╔══════════════════════════════════════════════╗"
echo "║  Ferrum Lab Kit — Raspberry Pi install       ║"
echo "║  Synaptic Four · synapticfour.com            ║"
echo "╚══════════════════════════════════════════════╝"
echo ""

arch="$(uname -m)"
case "$arch" in
  aarch64|arm64) ;;
  *)
    echo "warning: architecture is $arch (expected aarch64). Continuing anyway." >&2
    ;;
esac

if [[ -f /proc/meminfo ]]; then
  ram_mb="$(awk '/MemTotal/ {print int($2/1024)}' /proc/meminfo)"
  echo "==> Detected RAM: ${ram_mb} MB"
  if [[ "$ram_mb" -lt 3500 ]]; then
    echo "error: need at least ~4 GB RAM for field-edge (got ${ram_mb} MB)" >&2
    exit 1
  fi
  if [[ "$ram_mb" -lt 7000 ]] && grep -q 'solum-sidecar\|aai-broker' docker-compose.yml 2>/dev/null; then
    echo "warning: <8 GB RAM with Solum/ga4gh-infra — expect swap pressure; prefer Pi 5 8GB." >&2
  fi
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker is required. On Raspberry Pi OS / Ubuntu:" >&2
  echo "  curl -fsSL https://get.docker.com | sh" >&2
  echo "  sudo usermod -aG docker \"\$USER\"  # then log out/in" >&2
  exit 1
fi

if docker compose version >/dev/null 2>&1; then
  COMPOSE=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE=(docker-compose)
else
  echo "error: docker compose plugin not found" >&2
  exit 1
fi

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

if [[ -f deploy/docker-compose/ga4gh-infra-secrets/secrets.env ]]; then
  set -a
  # shellcheck disable=SC1091
  source deploy/docker-compose/ga4gh-infra-secrets/secrets.env
  set +a
fi

export FERRUM_IMAGE="${FERRUM_IMAGE:-__FERRUM_IMAGE_PIN__}"
export FERRUM_PORT="${FERRUM_PORT:-8080}"
DATA_DIR="${FERRUM_DATA_DIR:-$HOME/.ferrum}"
# Expand leading $HOME if present as literal from .env
DATA_DIR="${DATA_DIR/#\$HOME/$HOME}"
export FERRUM_DATA_DIR="$DATA_DIR"
mkdir -p "$DATA_DIR/objects"

echo "==> FERRUM_IMAGE=$FERRUM_IMAGE"
echo "==> Data directory: $DATA_DIR"
echo "==> Starting stack..."
"${COMPOSE[@]}" -f docker-compose.yml up -d --build

echo "==> Waiting for gateway on :${FERRUM_PORT}..."
ok=0
for i in $(seq 1 45); do
  if curl -fsS "http://127.0.0.1:${FERRUM_PORT}/health" >/dev/null 2>&1; then
    ok=1
    break
  fi
  sleep 2
done

if [[ "$ok" -ne 1 ]]; then
  echo "Gateway did not become healthy in time." >&2
  echo "Logs: ${COMPOSE[*]} -f docker-compose.yml logs ferrum-gateway" >&2
  exit 1
fi

echo ""
echo "Raspberry Pi field node is up."
echo "  Health:  http://127.0.0.1:${FERRUM_PORT}/health"
echo "  Beacon:  http://127.0.0.1:${FERRUM_PORT}/ga4gh/beacon/v2/info"
echo "  DRS:     http://127.0.0.1:${FERRUM_PORT}/ga4gh/drs/v1/service-info"
echo "  Data:    $DATA_DIR"
echo ""
"#
    .replace("__FERRUM_IMAGE_PIN__", pin)
}

#[cfg(test)]
mod tests {
    use lab_kit_core::parse_config;

    use super::*;

    #[test]
    fn raspberry_pi_bundle_writes_kit_files() {
        let raw = include_str!("../../../config/profiles/field-edge.toml");
        let cfg = parse_config(raw).unwrap();
        let fragments = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deploy/docker-compose");
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("pi-kit");
        generate_raspberry_pi_bundle(&cfg, &fragments, &out, &RaspberryPiBundleOptions::default())
            .unwrap();
        assert!(out.join("docker-compose.yml").is_file());
        assert!(out.join("lab-kit.toml").is_file());
        assert!(out.join(".env").is_file());
        assert!(out.join("README.md").is_file());
        assert!(out.join("install-on-pi.sh").is_file());
        assert!(out
            .join("deploy/docker-compose/docker-compose.gateway.yml")
            .is_file());
        assert!(out.join("deploy/docker-compose/edge.yml").is_file());
        let env = fs::read_to_string(out.join(".env")).unwrap();
        assert!(env.contains("FERRUM_IMAGE=ghcr.io/synapticfour/ferrum:"));
        assert!(!env.contains(":latest"));
        let compose = fs::read_to_string(out.join("docker-compose.yml")).unwrap();
        assert!(compose.contains("ferrum-gateway"));
        assert!(compose.contains("ENABLE_BEACON"));
    }

    #[test]
    fn raspberry_pi_bundle_with_solum_copies_dockerfile() {
        let raw = include_str!("../../../config/profiles/field-edge+solum.toml");
        let cfg = parse_config(raw).unwrap();
        let fragments = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deploy/docker-compose");
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("pi-kit");
        generate_raspberry_pi_bundle(&cfg, &fragments, &out, &RaspberryPiBundleOptions::default())
            .unwrap();
        assert!(out.join("deploy/docker-compose/solum.yml").is_file());
        assert!(out
            .join("deploy/docker-compose/Dockerfile.solum-sidecar")
            .is_file());
        let env = fs::read_to_string(out.join(".env")).unwrap();
        assert!(env.contains("SOLUM_SIDECAR_TOKEN=solum-"));
        assert!(!env.contains("solum-lab-kit-local-token-change-me"));
        let compose = fs::read_to_string(out.join("docker-compose.yml")).unwrap();
        assert!(compose.contains("solum-sidecar"));
    }
}
