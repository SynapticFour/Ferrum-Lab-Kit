# Contributing

Ferrum Lab Kit is licensed under the **Business Source License 1.1** (BUSL-1.1), with **parameters and grant text aligned to [Ferrum](https://github.com/SynapticFour/Ferrum)** (product name and repo URL adapted); see [LICENSE](LICENSE).

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/spdx-rs.py . --license BUSL-1.1 --check
```

New first-party Rust files start with `// SPDX-License-Identifier: BUSL-1.1`.

Or `./install.sh` for a release build of the `lab-kit` binary (see [README](README.md#install-cli-optional)).

To bump the **[Ferrum](https://github.com/SynapticFour/Ferrum)** `ferrum-core` pin: `./scripts/bump-ferrum.sh` (see [docs/FERRUM-INTEGRATION.md](docs/FERRUM-INTEGRATION.md)).

Optional **Postgres metadata** integration test (Docker; not part of default CI):

```bash
cargo test -p lab-kit-adapters --features integration-tests postgres_metadata_roundtrip
```

## Scope

- **Do not** re-implement GA4GH service logic here — integrate [Ferrum](https://github.com/SynapticFour/Ferrum) crates.
- **Docs:** tutorials, checklists, architecture explanations, and reference Compose belong here; link to Ferrum for product behaviour. Start from the [documentation index](docs/README.md).
- Prefer **Rust** for tooling; keep shell to trivial bootstrap only.
- Open-core boundary: **PDF conformance reports** are license-gated via a **signed** `FERRUM_LAB_KIT_LICENSE_KEY` (`flk1.<payload>.<sig>`) plus `lab-kit license activate`; JSON and GA4GH deployments are not.

## Identity

Commits and GitHub identity for this repository should use a **Synaptic Four** or **institutional** address. Do not use personal Gmail for project commits.

## Pull requests

1. One logical change per PR.
2. Update docs when changing `lab-kit.toml` schema or CLI commands.
3. CI must pass (fmt, clippy `-D warnings`, tests).
