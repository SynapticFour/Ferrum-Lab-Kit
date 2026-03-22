# Ferrum Lab Kit — documentation index

## Concepts & reproducibility

| Doc | What it’s for |
|-----|----------------|
| [GA4GH-WORKFLOW-PRIMER.md](GA4GH-WORKFLOW-PRIMER.md) | **TRS → WES → TES → engine** control flow, **DRS** metadata vs bytes, **nested Docker** paths, **WDL / CWL / Nextflow** at a glance, **amd64 vs arm64**, happy-path steps — patterns + links to **[Ferrum `GA4GH.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md)** (authoritative endpoints) |
| [GA4GH-STANDARDS.md](GA4GH-STANDARDS.md) | Short mapping of GA4GH surfaces to lab use cases |
| [OPERATIONS-CHECKLIST.md](OPERATIONS-CHECKLIST.md) | **Runbook checklist**: env vars, Docker, network, storage, naming |

## Architecture & deployment

| Doc | What it’s for |
|-----|----------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Lab Kit crates and how config drives deploy artefacts |
| [DEPLOYMENT-TARGETS.md](DEPLOYMENT-TARGETS.md) | Where Lab Kit aims to run (compose, K8s, systemd, …) |
| [BRING-YOUR-OWN.md](BRING-YOUR-OWN.md) | External endpoints vs generated stacks |

## Integration & identity

| Doc | What it’s for |
|-----|----------------|
| [FERRUM-INTEGRATION.md](FERRUM-INTEGRATION.md) | Git pin to `ferrum-core`, **`lab-kit ingest`**, gateway URL / token |
| [FERRUM-GA4GH-DEMO-OVERLAY.md](FERRUM-GA4GH-DEMO-OVERLAY.md) | Ferrum **GA4GH Compose overlay** (WES/TES Docker, workdirs, Crypt4GH) — mirrors under `contrib/ferrum/` + patch |
| [ELIXIR-AAI.md](ELIXIR-AAI.md) | LS Login / Passport-oriented notes |

## Conformance & governance

| Doc | What it’s for |
|-----|----------------|
| [CONFORMANCE.md](CONFORMANCE.md) | HelixTest, reports, JSON vs PDF |
| [BUSINESS-MODEL.md](BUSINESS-MODEL.md) | Open-core boundary |

## Examples in the repo

| Path | What it’s for |
|------|----------------|
| `config/lab-kit.example.toml` | Annotated full config template |
| `config/profiles/*.toml` | Ready-made service selections |
| `config/examples/ingest-register.json` | Sample body for **`lab-kit ingest register`** |

---

**Full platform (implementations, demos, engine details):** upstream **[SynapticFour/Ferrum](https://github.com/SynapticFour/Ferrum)** — e.g. `docs/WORKFLOWS.md`, `docs/INGEST-LAB-KIT.md`.
