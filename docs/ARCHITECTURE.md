# Architecture

**Ferrum** is the full sovereign bioinformatics platform (GA4GH implementations in Rust crates).

**Ferrum Lab Kit** is a **separate** repository: a deployment and integration layer that depends on Ferrum as a library, **without** forking or duplicating GA4GH service logic.

## Crates

| Crate | Role |
|-------|------|
| `lab-kit-core` | `lab-kit.toml` schema, validation, `ServiceRegistry`, `HealthAggregator` |
| `lab-kit-ferrum` | Git-pinned `ferrum-core` from [SynapticFour/Ferrum](https://github.com/SynapticFour/Ferrum) — shared types / future gateway glue |
| `lab-kit-auth` | ELIXIR LS Login OIDC (discovery, JWKS, JWT validation), `AuthProvider` trait, Passport / Beacon tier helpers |
| `lab-kit-adapters` | `StorageBackend`, `ComputeBackend`, `MetadataStore`, `WorkflowEngine` + S3, POSIX, SLURM, SQLite/Postgres (sqlx) |
| `lab-kit-deploy` | Compose merge, Helm values emission, systemd unit stubs |
| `lab-kit-report` | HelixTest JSON → `conformance-report.json`; PDF behind `FERRUM_LAB_KIT_LICENSE_KEY` |
| `lab-kit-selector` | `lab-kit` CLI: `init`, `generate`, `status`, `conformance` |

## Data flow

Configuration drives a **service registry** (deploy vs external URL). Generators emit artifacts; **Ferrum** binaries/images perform protocol work at runtime.

See also [BUSINESS-MODEL.md](BUSINESS-MODEL.md) for the open-core boundary.
