#!/usr/bin/env bash
# Bootstrap: build (and optionally install) the `lab-kit` CLI.
# Requires Rust / Cargo (https://rustup.rs). Respects rust-toolchain.toml in this repo.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./install.sh [options]

  --install       Also run `cargo install` so `lab-kit` is on PATH (default prefix: ~/.cargo)
  --prefix DIR    With --install: set CARGO_INSTALL_ROOT (binaries under DIR/bin)
  -h, --help      Show this help

Examples:
  ./install.sh
  ./install.sh --install
  ./install.sh --install --prefix "$HOME/.local"
EOF
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

DO_INSTALL=0
INSTALL_PREFIX=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install) DO_INSTALL=1; shift ;;
    --prefix)
      if [[ $# -lt 2 ]]; then echo "error: --prefix needs a directory" >&2; exit 1; fi
      INSTALL_PREFIX="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found. Install Rust: https://rustup.rs" >&2
  exit 1
fi

echo "==> Building lab-kit-selector (release)..."
cargo build --release -p lab-kit-selector --locked

BIN="$ROOT/target/release/lab-kit"
if [[ -x "$BIN" ]]; then
  echo "==> Binary: $BIN"
  "$BIN" --version || true
else
  echo "error: expected binary missing: $BIN" >&2
  exit 1
fi

if [[ "$DO_INSTALL" -eq 1 ]]; then
  export CARGO_INSTALL_ROOT="${INSTALL_PREFIX:-${CARGO_INSTALL_ROOT:-$HOME/.cargo}}"
  echo "==> Installing lab-kit to \$CARGO_INSTALL_ROOT/bin ($CARGO_INSTALL_ROOT/bin)"
  cargo install --path crates/lab-kit-selector --locked --force
  echo "==> Done. Ensure $CARGO_INSTALL_ROOT/bin is on your PATH."
fi
