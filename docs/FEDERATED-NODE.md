# Federated node (technical capability)

`lab-kit init --profile federated-node --non-interactive` generates a **technical** GA4GH node:

| Surface | Out of the box |
|---------|----------------|
| Beacon v2 | boolean + count (`record` is rejected by Ferrum) |
| DRS | POSIX/SQLite, `GET …/service-info` |
| Passport-on-DRS | ga4gh-infra broker + clearinghouse; Ferrum does **not** mint Passports |
| service-info | each enabled Ferrum surface |
| TES / WES | **off** — set `tes = true` / `wes = true` in the profile to opt in |
| TLS termination | Traefik sidecar on **8443** (default cert; operator replaces it) |
| Registry entry | `FERRUM_DISCOVERY__AUTO_REGISTER` → local service-registry |

HelixTest against the same surfaces: `helixtest --all --mode ferrum+infra --profile ferrum-infra --fail-level 2` (needs a live stack + `lab-kit generate infra-secrets`). Ferrum’s scheduled `ferrum+infra` job is the live proof against the Ferrum pilot; Lab Kit CI **generates** this compose and asserts the flags.

## This is not a membership card

Compose-up is **not**:

- an **EGA Federated Node**
- a **GHGA** site
- a **GDI** national node
- an **ELIXIR** AAI / Beacon-Network join

Those are **operator onboarding**: contracts, IdP registration, network peering. Lab Kit delivers the node. `gdi-national-node` and `full-elixir-node` remain LS-Login + Beacon-Network **stubs** for that paperwork — not OOTB proof.

## Operator checklist (docs-check, not a badge)

Copy into your runbook. Empty boxes are **your** work, not a Lab Kit certification.

| Check | Owner |
|-------|--------|
| ☐ Beacon `/info` and boolean+count query | Lab Kit profile |
| ☐ DRS service-info + object GET | Lab Kit profile |
| ☐ Passport presented to DRS (broker login) | ga4gh-infra co-deploy |
| ☐ Service registered in the local registry | discovery auto-register |
| ☐ TLS: replace Traefik default cert / ACME; clients trust the CA | operator |
| ☐ LS Login / institutional IdP (not the mock IdP) | operator |
| ☐ Beacon Network join (ELIXIR) | operator + ELIXIR |
| ☐ GHGA / EGA membership, DAC, AAI contract | those organisations |
| ☐ GDI / ELIXIR national-node checklists | those programmes |

See also [OPERATIONS-CHECKLIST.md](OPERATIONS-CHECKLIST.md) (TLS / trust) and [ELIXIR-AAI.md](ELIXIR-AAI.md).

## Quick start

```bash
lab-kit init --profile federated-node --non-interactive
lab-kit generate infra-secrets
lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose -o docker-compose.yml
# copies tls-traefik-dynamic.yaml next to the compose file
docker compose -f docker-compose.yml up -d
# HTTP  :8080  HTTPS :8443
```

Optional compute: edit `lab-kit.toml` `[services]` `wes` / `tes` to `true` and regenerate (full Ferrum image, not `-edge-infra`).
