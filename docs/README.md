# Ferrum Lab Kit — documentation index

Every document under `docs/`, grouped as a tutorial, how-to guides, reference, and explanation. One line each.

## Tutorial

| Document | What it is |
|----------|------------|
| [GETTING-STARTED.md](GETTING-STARTED.md) | Clone, `make up`, and the optional CLI. |

## How-to guides

| Document | What it is |
|----------|------------|
| [RASPBERRY-PI.md](RASPBERRY-PI.md) | Generate a Pi field kit and install it on the device. |
| [OPERATIONS-CHECKLIST.md](OPERATIONS-CHECKLIST.md) | Environment, Docker, network, storage, and names. |
| [BRING-YOUR-OWN.md](BRING-YOUR-OWN.md) | Point at endpoints you already run, instead of a generated stack. |
| [SOLUM-CO-DEPLOY.md](SOLUM-CO-DEPLOY.md) | Optional Solum sidecar. Consent stays in Solum. |
| [BRA-CO-DEPLOY.md](BRA-CO-DEPLOY.md) | Optional BioResearch Assistant companion. Not a combined product. |
| [FEDERATED-NODE.md](FEDERATED-NODE.md) | Beacon, DRS, infra, and TLS for a node. Not a membership badge. |
| [FERRUM-GA4GH-DEMO-OVERLAY.md](FERRUM-GA4GH-DEMO-OVERLAY.md) | Compose overlay shared with the laptop demo. |
| [ELIXIR-AAI.md](ELIXIR-AAI.md) | Notes toward LS Login and Passports. |
| [GA4GH-WORKFLOW-PRIMER.md](GA4GH-WORKFLOW-PRIMER.md) | TRS, WES, TES, and engines, with links to Ferrum for the endpoints. |
| [CONFORMANCE.md](CONFORMANCE.md) | HelixTest reports. JSON is open. PDF needs the licence token. |

## Reference

| Document | What it is |
|----------|------------|
| [IMAGE-PIN-POLICY.md](IMAGE-PIN-POLICY.md) | When a floating tag is refused, and how pins are set. |
| [FERRUM-INTEGRATION.md](FERRUM-INTEGRATION.md) | The git pin, the image variants, and `lab-kit ingest`. |
| [IDENTITY.md](IDENTITY.md) | Names and versions this repository uses for itself. |
| [GA4GH-STANDARDS.md](GA4GH-STANDARDS.md) | Short map from a GA4GH surface to a lab task. |
| [ECOSYSTEM.md](ECOSYSTEM.md) | This repository beside Ferrum, ga4gh-infra, and the demo. |
| [BUSINESS-MODEL.md](BUSINESS-MODEL.md) | Open-core boundary. Not a price list. |

## Explanation

| Document | What it is |
|----------|------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crates, and how configuration becomes start files. |
| [FOR-EVALUATORS.md](FOR-EVALUATORS.md) | Status, the deliberate v0.3.2 pin, and what this page does not record. |
| [DEPLOYMENT-TARGETS.md](DEPLOYMENT-TARGETS.md) | Compose, Kubernetes, systemd, and the field board as targets. |

## Examples in the repo

| Path | What it is for |
|------|----------------|
| `config/lab-kit.example.toml` | Annotated full config template |
| `config/profiles/*.toml` | Ready-made selections (`field-edge`, `+infra`, `+solum`, …) |
| `.env.example` | Compose / CLI environment variables |
| `config/examples/ingest-register.json` | Sample body for **`lab-kit ingest register`** |

---

The implementations and the engine details are in [Ferrum](https://github.com/SynapticFour/Ferrum), for example `docs/WORKFLOWS.md` and `docs/INGEST-LAB-KIT.md`.

MII/KDS operations, when needed, are optional wrappers here (`lab-kit mii ...`) and run in Ferrum (`ferrum mii ...`).

*[← Back to the README](../README.md)*
