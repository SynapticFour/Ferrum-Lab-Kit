#!/usr/bin/env bash
# Ferrum Lab Kit — Field/Edge one-shot installer.
# Targets Raspberry Pi OS (ARM64), Ubuntu 22.04/24.04 (x86_64 / ARM64).
# Minimal GA4GH node: Beacon v2 + DRS, SQLite + local filesystem (no PostgreSQL/MinIO).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

LAB_KIT_VERSION="${LAB_KIT_VERSION:-latest}"
LAB_KIT_REPO="${LAB_KIT_REPO:-SynapticFour/Ferrum-Lab-Kit}"
BEACON_PORT="${BEACON_PORT:-8080}"
DATA_DIR="${FERRUM_DATA_DIR:-$HOME/.ferrum}"
CONFIG="${LAB_KIT_CONFIG:-lab-kit.toml}"
COMPOSE_OUT="${COMPOSE_OUT:-docker-compose.yml}"
WITH_INFRA=0

usage() {
  cat <<'EOF'
Usage: install-edge.sh [--with-infra]

  --with-infra   Co-deploy ga4gh-infra auth plane (broker 8180, registry 8183, mock-idp 9100)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --with-infra)
      WITH_INFRA=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

echo "╔══════════════════════════════════════════════╗"
echo "║  Ferrum Lab Kit — Field/Edge Setup           ║"
echo "║  Synaptic Four · synapticfour.com            ║"
echo "╚══════════════════════════════════════════════╝"
echo ""
echo "This script installs a minimal GA4GH-compatible local node"
echo "(Beacon v2 + DRS) on this machine. No cloud required."
echo ""

require_root_for_docker() {
  if command -v docker >/dev/null 2>&1; then
    return 0
  fi
  if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
    echo "Docker is not installed. Re-run with sudo to install Docker, or install Docker first." >&2
    exit 1
  fi
}

detect_arch() {
  local machine
  machine="$(uname -m)"
  case "$machine" in
    x86_64|amd64) echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *)
      echo "error: unsupported architecture: $machine (need x86_64 or aarch64)" >&2
      exit 1
      ;;
  esac
}

install_docker_apt() {
  if command -v docker >/dev/null 2>&1; then
    echo "==> Docker already installed: $(docker --version)"
    return 0
  fi

  echo "==> Installing Docker via apt..."
  require_root_for_docker

  apt-get update -qq
  apt-get install -y ca-certificates curl gnupg
  install -m 0755 -d /etc/apt/keyrings
  if [[ ! -f /etc/apt/keyrings/docker.gpg ]]; then
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    chmod a+r /etc/apt/keyrings/docker.gpg
  fi

  local distro codename
  if [[ -f /etc/os-release ]]; then
    # shellcheck source=/dev/null
    . /etc/os-release
    distro="${ID:-ubuntu}"
    codename="${VERSION_CODENAME:-jammy}"
  else
    distro="ubuntu"
    codename="jammy"
  fi

  # Raspberry Pi OS reports debian; map to bookworm for Docker CE repo when needed.
  if [[ "$distro" == "raspbian" || "$distro" == "debian" ]]; then
    distro="debian"
    codename="${VERSION_CODENAME:-bookworm}"
  fi

  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/${distro} ${codename} stable" \
    > /etc/apt/sources.list.d/docker.list

  apt-get update -qq
  apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
  systemctl enable --now docker 2>/dev/null || true
  echo "==> Docker installed."
}

install_lab_kit_binary() {
  local arch bin_name dest
  arch="$(detect_arch)"
  bin_name="lab-kit-${arch}"
  dest="${INSTALL_DIR:-$HOME/.local/bin}/lab-kit"

  mkdir -p "$(dirname "$dest")"

  # Offline / bundled binary next to this script (e.g. USB stick deployment).
  if [[ -x "$ROOT/bin/lab-kit" ]]; then
    echo "==> Using bundled lab-kit: $ROOT/bin/lab-kit"
    install -m 0755 "$ROOT/bin/lab-kit" "$dest"
    echo "$dest"
    return 0
  fi
  if [[ -x "$ROOT/bin/$bin_name" ]]; then
    echo "==> Using bundled $bin_name"
    install -m 0755 "$ROOT/bin/$bin_name" "$dest"
    echo "$dest"
    return 0
  fi

  # Build from source when cargo is available (developer / air-gapped clone).
  if command -v cargo >/dev/null 2>&1 && [[ -f "$ROOT/Cargo.toml" ]]; then
    echo "==> Building lab-kit from source..."
    cargo build --release -p lab-kit-selector --locked
    install -m 0755 "$ROOT/target/release/lab-kit" "$dest"
    echo "$dest"
    return 0
  fi

  echo "==> Downloading lab-kit release (${LAB_KIT_VERSION}, ${arch})..."
  local url tmp
  tmp="$(mktemp)"
  if [[ "$LAB_KIT_VERSION" == "latest" ]]; then
    url="https://github.com/${LAB_KIT_REPO}/releases/latest/download/${bin_name}"
  else
    url="https://github.com/${LAB_KIT_REPO}/releases/download/${LAB_KIT_VERSION}/${bin_name}"
  fi

  if curl -fsSL "$url" -o "$tmp"; then
    install -m 0755 "$tmp" "$dest"
    rm -f "$tmp"
    echo "$dest"
    return 0
  fi

  rm -f "$tmp"
  echo "error: could not download lab-kit. Place a binary at bin/lab-kit or install Rust and clone this repo." >&2
  exit 1
}

ensure_compose_cmd() {
  if docker compose version >/dev/null 2>&1; then
    COMPOSE=(docker compose)
  elif command -v docker-compose >/dev/null 2>&1; then
    COMPOSE=(docker-compose)
  else
    echo "error: docker compose plugin not found" >&2
    exit 1
  fi
}

wait_for_beacon() {
  local url="http://127.0.0.1:${BEACON_PORT}/ga4gh/beacon/v2/info"
  local i
  for i in $(seq 1 30); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  # Fallback: gateway health on same port (edge overlay).
  if curl -fsS "http://127.0.0.1:${BEACON_PORT}/health" >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

main() {
  install_docker_apt
  ensure_compose_cmd

  export FERRUM_DATA_DIR="$DATA_DIR"
  mkdir -p "$DATA_DIR/objects"

  local lab_kit
  lab_kit="$(install_lab_kit_binary)"
  export PATH="$(dirname "$lab_kit"):$PATH"

  if [[ "$WITH_INFRA" -eq 1 ]]; then
    echo "==> Initializing field-edge+infra profile (Ferrum + ga4gh-infra co-deploy)..."
    "$lab_kit" init --profile field-edge+infra --non-interactive --output "$CONFIG" --data-dir "$DATA_DIR"
  else
    echo "==> Initializing field-edge profile..."
    "$lab_kit" init --profile field-edge --non-interactive --output "$CONFIG" --data-dir "$DATA_DIR"
  fi

  echo "==> Generating docker-compose.yml..."
  compose_args=(
    generate compose
    --config "$CONFIG"
    --fragments deploy/docker-compose
    --output "$COMPOSE_OUT"
  )
  if [[ "$WITH_INFRA" -eq 1 ]]; then
    compose_args+=(--with-ga4gh-infra)
  fi
  "$lab_kit" "${compose_args[@]}"

  echo "==> Starting Ferrum Lab Kit stack..."
  "${COMPOSE[@]}" -f "$COMPOSE_OUT" up -d

  echo "==> Waiting for Beacon v2 on port ${BEACON_PORT}..."
  if wait_for_beacon; then
    echo ""
    echo "╔══════════════════════════════════════════════╗"
    echo "║  Field/Edge node is running                  ║"
    echo "╚══════════════════════════════════════════════╝"
    echo ""
    echo "  Beacon v2 info:  http://127.0.0.1:${BEACON_PORT}/ga4gh/beacon/v2/info"
    echo "  Data directory:  ${DATA_DIR}"
    echo "  Config:          ${ROOT}/${CONFIG}"
    echo ""
    echo "  Conformance:  lab-kit conformance run --config ${CONFIG}"
    echo "  Status:       lab-kit status --config ${CONFIG}"
    echo ""
  else
    echo ""
    echo "Stack started but Beacon did not respond on port ${BEACON_PORT} within 60s." >&2
    echo "Images may still be placeholders — check: ${COMPOSE[*]} -f ${COMPOSE_OUT} ps" >&2
    exit 1
  fi
}

main
