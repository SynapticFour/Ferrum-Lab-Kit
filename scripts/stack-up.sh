#!/usr/bin/env bash
# Start the Ferrum Lab Kit stack. Runs install-edge.sh on first use, then reuses compose.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

COMPOSE_OUT="${COMPOSE_OUT:-docker-compose.yml}"
WITH_INFRA=0

usage() {
  cat <<'EOF'
Usage: scripts/stack-up.sh [--with-infra]

  --with-infra   Co-deploy ga4gh-infra auth plane (broker, registry, mock-idp)

If docker-compose.yml is missing, delegates to ./install-edge.sh (full first-time setup).
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    --with-infra)
      WITH_INFRA=1
      shift
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ ! -f "$COMPOSE_OUT" ]]; then
  args=()
  if [[ "$WITH_INFRA" -eq 1 ]]; then
    args+=(--with-infra)
  fi
  exec "$ROOT/install-edge.sh" "${args[@]}"
fi

if [[ "$WITH_INFRA" -eq 1 ]] && ! grep -q 'ga4gh-infra\|8180' "$COMPOSE_OUT" 2>/dev/null; then
  echo "Existing $COMPOSE_OUT does not include ga4gh-infra. Re-run:" >&2
  echo "  make destroy && make up-with-infra" >&2
  exit 1
fi

echo "==> Starting Ferrum Lab Kit stack ($COMPOSE_OUT)..."
docker compose -f "$COMPOSE_OUT" up -d --build
echo "Stack is up. Status: lab-kit status --config ${LAB_KIT_CONFIG:-lab-kit.toml}"
