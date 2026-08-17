# Solum co-deploy (optional companion)

Lab Kit can **wire** a [Solum](https://github.com/SynapticFour/Solum) sidecar next to Ferrum so operators get purpose-bound consent checks from day one (Ferrum H2.1 / Showcase [ADR 0001](https://github.com/SynapticFour/SynapticFour-Showcase/blob/main/docs/adr/0001-solum-ferrum-consent-access.md)).

**Ownership stays in Solum.** Lab Kit does not implement jurisdiction profiles, FHIR/openEHR, or EHRbase. It only:

1. Merges `deploy/docker-compose/solum.yml` (build from pinned Solum tag or `SOLUM_IMAGE`)
2. Sets `FERRUM_SOLUM__BASE_URL` / `FERRUM_SOLUM__SIDECAR_TOKEN` on `ferrum-gateway`

## When to use it

| Need | Use |
|------|-----|
| Genomic GA4GH only | Lab Kit without Solum |
| Ferrum + consent teeth from start | `--with-solum` or profile `field-edge+solum` |
| Full clinical compliance / Track B CDR | Solum + [Solum-Demo](https://github.com/SynapticFour/Solum-Demo); not Lab Kit |
| Multi-product evidence chain | [SynapticFour-Showcase](https://github.com/SynapticFour/SynapticFour-Showcase) |

## Quick start

```bash
# Profile (recommended)
lab-kit init --profile field-edge+solum --non-interactive
lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose -o docker-compose.yml
docker compose -f docker-compose.yml up -d --build

# Or flag on any config
lab-kit generate compose --config lab-kit.toml --with-solum -o docker-compose.yml

# Make shortcuts
make up-with-solum
make up-with-infra-solum   # ga4gh-infra + Solum
```

## Config (`lab-kit.toml`)

```toml
[solum]
enabled = true
port = 8787
# sidecar_token = "change-me"   # prefer SOLUM_SIDECAR_TOKEN in .env
# default_subject = "patient-1"
# default_purpose = "research"
timeout_secs = 5
solum_tag = "v0.1.0"
```

Env overrides: see [`.env.example`](../.env.example) (`SOLUM_*`, `FERRUM_SOLUM_*`).

## Ports

| Service | Host port |
|---------|-----------|
| Ferrum gateway | **8080** |
| Solum sidecar | **8787** |

Do not map a Solum demo dashboard onto 8080 while Ferrum owns that port.

## Build note

There is **no** required published Solum GHCR image for Lab Kit. Default compose **builds** `Dockerfile.solum-sidecar` (clones the pinned Solum tag). First `docker compose up --build` needs network access. Override with `SOLUM_IMAGE=…` when you have a prebuilt image.

## Related docs

- Solum ↔ Ferrum: [Solum `docs/ferrum.md`](https://github.com/SynapticFour/Solum/blob/main/docs/ferrum.md)
- Sidecar contract: [Solum SIDECAR-INTEGRATION](https://github.com/SynapticFour/Solum/blob/main/docs/customer/SIDECAR-INTEGRATION.md)
- Lab Kit ecosystem map: [ECOSYSTEM.md](ECOSYSTEM.md)
