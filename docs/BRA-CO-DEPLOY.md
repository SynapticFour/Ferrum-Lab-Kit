# BRA co-deploy (optional companion)

Lab Kit can **wire** [BioResearch Assistant](https://github.com/SynapticFour/bioresearch-assistant) next to Ferrum so the workbench talks to **institutional** DRS/WES.

**Ownership stays in BRA.** Lab Kit does not implement Phenopackets, literature, or a second archive. It only:

1. Merges `deploy/docker-compose/bra.yml`
2. Sets `FERRUM_DRS_URL` / `FERRUM_WES_URL` on the BRA container to `ferrum-gateway`

This is **not** a fifth product and **not** a Ferrum+BRA combo SKU. BRA remains its own license.

## When to use it

| Need | Use |
|------|-----|
| Genomic GA4GH only | Lab Kit without BRA |
| Ferrum archive + researcher UI | `--with-bra` or profile `bra-companion` |
| BRA without Ferrum | Run BRA standalone (it has its own DRS/WES). Do not use this profile. |
| Consent sidecar | [SOLUM-CO-DEPLOY.md](SOLUM-CO-DEPLOY.md) — separate companion |

**One institute, one DRS.** With these URLs set, BRA is a GA4GH **client**. Do not treat BRA’s local DRS as a second institutional archive.

## Quick start

```bash
# Profile (DRS + WES + BRA client env)
lab-kit init --profile bra-companion --non-interactive
export BRA_IMAGE=…   # required — Lab Kit does not publish a BRA image
lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose -o docker-compose.yml
docker compose -f docker-compose.yml up -d

# Or flag on any config that already has DRS/WES
lab-kit generate compose --config lab-kit.toml --with-bra -o docker-compose.yml
```

Pin BRA **v0.2.1** (product tag). Build the image from that tag, or point `BRA_IMAGE` at whatever you already run.

HTTP: published GA4GH DRS/WES OpenAPI. Ferrum [utoipa dump](https://github.com/SynapticFour/Ferrum/blob/main/docs/openapi/ferrum.openapi.json) only for Ferrum-only paths.

## Config (`lab-kit.toml`)

```toml
[bra]
enabled = true
port = 5173
bra_tag = "v0.2.1"
```

Env: see [`.env.example`](../.env.example) (`BRA_IMAGE`, `BRA_PORT`, `BRA_OIDC_*`, `FERRUM_BEARER_TOKEN`).

## Ports

| Service | Host port |
|---------|-----------|
| Ferrum gateway | **8080** |
| BRA UI (if the image exposes it) | **5173** |

## What this is not

- Not a published `ghcr.io/synapticfour/bioresearch-assistant` image from Lab Kit
- Not Passport issuance (point `BRA_OIDC_ISSUER` at ga4gh-infra if you use `--with-ga4gh-infra`)
- Not Solum consent (use `[solum]` separately)

## Related docs

- BRA identity: [bioresearch-assistant IDENTITY](https://github.com/SynapticFour/bioresearch-assistant/blob/main/docs/IDENTITY.md)
- GA4GH specs Ferrum implements: [Ferrum GA4GH.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md)
- Ferrum utoipa dump (not a replacement spec): [Ferrum openapi README](https://github.com/SynapticFour/Ferrum/blob/main/docs/openapi/README.md)
- Lab Kit ecosystem map: [ECOSYSTEM.md](ECOSYSTEM.md)
