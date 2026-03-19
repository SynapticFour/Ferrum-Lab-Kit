# Contributing

Ferrum Lab Kit is licensed under the **Business Source License 1.1** (BUSL-1.1), with **parameters and grant text aligned to [Ferrum](https://github.com/SynapticFour/Ferrum)** (product name and repo URL adapted); see [LICENSE](LICENSE).

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Optional **Postgres metadata** integration test (Docker; not part of default CI):

```bash
cargo test -p lab-kit-adapters --features integration-tests postgres_metadata_roundtrip
```

## Scope

- **Do not** re-implement GA4GH service logic here — integrate [Ferrum](https://github.com/SynapticFour/Ferrum) crates.
- Prefer **Rust** for tooling; keep shell to trivial bootstrap only.
- Open-core boundary: **PDF conformance reports** are license-gated at runtime via `FERRUM_LAB_KIT_LICENSE_KEY`; JSON and GA4GH deployments are not.

## Pull requests

1. One logical change per PR.
2. Update docs when changing `lab-kit.toml` schema or CLI commands.
3. CI must pass (fmt, clippy `-D warnings`, tests).
