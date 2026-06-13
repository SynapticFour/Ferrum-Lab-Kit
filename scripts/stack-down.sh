#!/usr/bin/env bash
# Stop or remove the Ferrum Lab Kit Docker stack (standalone or co-deploy with ga4gh-infra).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

COMPOSE_OUT="${COMPOSE_OUT:-docker-compose.yml}"
REMOVE_VOLUMES=0

usage() {
  cat <<'EOF'
Usage: scripts/stack-down.sh [--volumes]

  --volumes   Remove Docker volumes (SQLite/Postgres data, object store, etc.)

Environment:
  COMPOSE_OUT   Compose file path (default: docker-compose.yml)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    --volumes|-v)
      REMOVE_VOLUMES=1
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
  echo "No compose file at $COMPOSE_OUT — nothing to stop." >&2
  exit 0
fi

down_args=(down --remove-orphans)
if [[ "$REMOVE_VOLUMES" -eq 1 ]]; then
  down_args+=(-v)
fi

docker compose -f "$COMPOSE_OUT" "${down_args[@]}"

if [[ "$REMOVE_VOLUMES" -eq 1 ]]; then
  echo "Ferrum Lab Kit stack destroyed (volumes removed)."
else
  echo "Ferrum Lab Kit stack stopped (volumes kept)."
fi
