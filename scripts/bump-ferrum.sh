#!/usr/bin/env bash
# Bump the Ferrum git pin (ferrum-core) to latest main or a given full SHA.
# Updates: crates/lab-kit-ferrum/Cargo.toml, crates/lab-kit-ferrum/src/lib.rs, config/ci/ferrum-revision.txt
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

# perl in-place edit: works the same on macOS and Linux
perl -i -pe "s/rev = \"[0-9a-f]{40}\"/rev = \"$FERRUM_REV\"/" \
  "$ROOT/crates/lab-kit-ferrum/Cargo.toml"

perl -i -pe "s/pub const FERRUM_GIT_REV: &str = \"[0-9a-f]{40}\"/pub const FERRUM_GIT_REV: &str = \"$FERRUM_REV\"/" \
  "$ROOT/crates/lab-kit-ferrum/src/lib.rs"

TMP="$(mktemp)"
awk -v sha="$FERRUM_REV" '
  /^[0-9a-f]{40}$/ { print sha; replaced = 1; next }
  { print }
  END { if (!replaced) print sha }
' "$ROOT/config/ci/ferrum-revision.txt" >"$TMP"
mv "$TMP" "$ROOT/config/ci/ferrum-revision.txt"

echo "Updated:"
echo "  - crates/lab-kit-ferrum/Cargo.toml"
echo "  - crates/lab-kit-ferrum/src/lib.rs (FERRUM_GIT_REV)"
echo "  - config/ci/ferrum-revision.txt"
echo ""
echo "Next:"
echo "  cargo update -p ferrum-core"
echo "  cargo test --workspace"
