#!/usr/bin/env bash
# Bump the Ferrum git pin (ferrum-core) to latest main or a given full SHA.
# Updates: crates/lab-kit-ferrum/Cargo.toml, config/ci/ferrum-revision.txt,
#          config/ci/ferrum-image.txt, ferrum-image-edge.txt,
#          ferrum-image-edge-infra.txt, ferrum-image-arm64.txt
#
# Usage:
#   ./scripts/bump-ferrum.sh              # use origin/main tip
#   ./scripts/bump-ferrum.sh <40-char-sha> # pin exact commit
#   ./scripts/bump-ferrum.sh --dry-run    # show SHA only, do not write files
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FERRUM_REMOTE="${FERRUM_REMOTE:-https://github.com/SynapticFour/Ferrum.git}"

usage() {
  cat <<'EOF'
Usage: ./scripts/bump-ferrum.sh [--dry-run] [<full-40-hex-sha>]

  --dry-run   Print resolved revision and exit without editing files.
  <sha>       Pin this commit (40 lowercase hex chars). Otherwise uses refs/heads/main.

Environment:
  FERRUM_REMOTE   Git URL (default: https://github.com/SynapticFour/Ferrum.git)

After bumping:
  cargo update -p ferrum-core
  cargo test --workspace
EOF
}

DRY_RUN=0
SHA_ARG=""

for a in "$@"; do
  case "$a" in
    -h | --help)
      usage
      exit 0
      ;;
    --dry-run)
      DRY_RUN=1
      ;;
    *)
      if [[ -n "$SHA_ARG" ]]; then
        echo "error: unexpected extra argument: $a" >&2
        usage >&2
        exit 1
      fi
      SHA_ARG="$a"
      ;;
  esac
done

resolve_sha() {
  if [[ -n "$SHA_ARG" ]]; then
    echo "$SHA_ARG"
    return
  fi
  git ls-remote "$FERRUM_REMOTE" refs/heads/main | awk '{ print $1; exit }'
}

FERRUM_REV="$(resolve_sha)"

if [[ -z "$FERRUM_REV" ]]; then
  echo "error: could not resolve Ferrum revision (git ls-remote failed?)" >&2
  exit 1
fi

if ! [[ "$FERRUM_REV" =~ ^[0-9a-f]{40}$ ]]; then
  echo "error: expected full 40-char lowercase hex SHA, got: $FERRUM_REV" >&2
  exit 1
fi

echo "Ferrum revision: $FERRUM_REV"

if [[ "$DRY_RUN" -eq 1 ]]; then
  exit 0
fi

OLD_SHA="$(
  grep -E 'rev = "[0-9a-f]{40}"' "$ROOT/crates/lab-kit-ferrum/Cargo.toml" \
    | sed -E 's/.*"([0-9a-f]{40})".*/\1/' | head -n1
)"

# perl in-place edit: works the same on macOS and Linux
perl -i -pe "s/rev = \"[0-9a-f]{40}\"/rev = \"$FERRUM_REV\"/" \
  "$ROOT/crates/lab-kit-ferrum/Cargo.toml"

TMP="$(mktemp)"
awk -v sha="$FERRUM_REV" '
  /^[0-9a-f]{40}$/ { print sha; replaced = 1; next }
  { print }
  END { if (!replaced) print sha }
' "$ROOT/config/ci/ferrum-revision.txt" >"$TMP"
mv "$TMP" "$ROOT/config/ci/ferrum-revision.txt"

write_image_pin() {
  local file="$1"
  local comment="$2"
  local image="$3"
  cat >"$file" <<EOF
$comment
$image
EOF
}

write_image_pin "$ROOT/config/ci/ferrum-image.txt" \
  "# Default Ferrum monolith image for generated Compose/Helm (full variant, unsuffixed SHA tag).
# Bumped with ./scripts/bump-ferrum.sh. Override with FERRUM_IMAGE." \
  "ghcr.io/synapticfour/ferrum:$FERRUM_REV"
write_image_pin "$ROOT/config/ci/ferrum-image-edge.txt" \
  "# Default Ferrum edge image (DRS + Beacon + htsget compiled in; Lab Kit disables unused routes).
# Bumped with ./scripts/bump-ferrum.sh. Override with FERRUM_IMAGE." \
  "ghcr.io/synapticfour/ferrum:$FERRUM_REV-edge"
write_image_pin "$ROOT/config/ci/ferrum-image-edge-infra.txt" \
  "# Ferrum edge + ga4gh-infra hooks (clearinghouse / discovery). Override with FERRUM_IMAGE.
# Bumped with ./scripts/bump-ferrum.sh." \
  "ghcr.io/synapticfour/ferrum:$FERRUM_REV-edge-infra"
write_image_pin "$ROOT/config/ci/ferrum-image-arm64.txt" \
  "# ARM64 Ferrum image for Pi kits (edge variant; same tag when GHCR is multi-arch).
# Override with FERRUM_IMAGE. Prefer lab-kit build image --platform linux/arm64 when the tag is amd64-only." \
  "ghcr.io/synapticfour/ferrum:$FERRUM_REV-edge"

if [[ -n "$OLD_SHA" && "$OLD_SHA" != "$FERRUM_REV" ]]; then
  for f in \
    "$ROOT/.env.example" \
    "$ROOT/deploy/docker-compose/docker-compose.gateway.yml" \
    "$ROOT/deploy/helm/values.yaml" \
    "$ROOT/install-edge.sh" \
    "$ROOT/crates/lab-kit-deploy/src/images.rs"
  do
    [[ -f "$f" ]] || continue
    perl -i -pe "s/$OLD_SHA/$FERRUM_REV/g" "$f"
  done
fi

echo "Updated:"
echo "  - crates/lab-kit-ferrum/Cargo.toml"
echo "  - config/ci/ferrum-revision.txt"
echo "  - config/ci/ferrum-image.txt (full)"
echo "  - config/ci/ferrum-image-edge.txt"
echo "  - config/ci/ferrum-image-edge-infra.txt"
echo "  - config/ci/ferrum-image-arm64.txt (edge)"
echo "  - operator defaults (.env.example, gateway compose, helm values, install-edge fallback)"
echo ""
echo "Next:"
echo "  cargo update -p ferrum-core"
echo "  cargo test --workspace"
