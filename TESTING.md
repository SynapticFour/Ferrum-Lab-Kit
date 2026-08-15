# Testing Strategy

## What CI actually gates

| Gate | Where | What must pass |
|------|--------|----------------|
| Format / Clippy / unit+integration tests | `.github/workflows/ci.yml` (`rust` job) | `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace` |
| cargo-deny | `ci.yml` (`rust` job) | `cargo deny check` (`deny.toml`) |
| Field-edge compose generation | same `rust` job (not a separate workflow job) | Non-interactive `init` + `generate compose` + YAML parse + `bash -n install-edge.sh` |
| aarch64 release build | `ci.yml` (`build-arm64` job) | `cargo build -p lab-kit-selector --release` on `ubuntu-24.04-arm` |
| Compose profile + HelixTest CLI smoke | `.github/workflows/conformance.yml` (push to `main` on relevant paths) | Generate compose for beacon-only / field-edge; build pinned HelixTest; `--version` + CLI smoke — **no live services** |
| Live HelixTest suite | `conformance.yml` **opt-in only** (`workflow_dispatch` + `run_live_suite=true`) | `helixtest --all … --only beacon` against generated compose — **fails the job on failure** (requires pullable Ferrum images) |
| CodeQL | `.github/workflows/codeql.yml` | Weekly / configured schedule; SARIF upload per `.github/ci-config.json` |
| Secret scan / dependency review | respective workflows | PRs/pushes; dependency-review is **blocking** |

Postgres adapter tests under `lab-kit-adapters` with `--features integration-tests` are **local/opt-in** (Docker required) and are not part of the default CI matrix.

## Minimum local checks before merging

```bash
./scripts/hooks/ci-check.sh   # or: pre-commit run --all-files
cargo test --workspace
cargo deny check              # if cargo-deny is installed
```

## PR requirement

Any non-trivial behavior change should include tests or a documented reason why tests are not feasible.
