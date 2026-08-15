# Image pin policy

**Status:** 2026-08-15
**Repo:** Ferrum-Lab-Kit

## Policy

| Context | Rule |
|---------|------|
| **Generated Compose / Helm / Pi kit / install-edge** | Use the SHA tag in `config/ci/ferrum-image.txt` (ARM64: `ferrum-image-arm64.txt`). Do **not** emit floating `:latest` / `:latest-arm64`. |
| **Pilot / production** | Prefer digest pins (`image@sha256:…`) or SemVer tags when Ferrum publishes them. Override with `FERRUM_IMAGE`. |
| **Third-party infra** (Traefik, Postgres, …) | Pin minor/major versions (e.g. `traefik:v3.3`); bump deliberately in a PR with notes. |
| **ga4gh-infra siblings** | Use version env vars (`MOCK_IDP_VERSION`, etc.) already present in compose; keep defaults aligned with published infra tags. |
| **Legacy per-service fragments** | `--legacy-per-service` still references unpublished `synapticfour/ferrum-*` images. Operators who use that path must supply their own tags. |
| **Solum sidecar** | Compose default `synapticfour/solum-sidecar:lab-kit` is a **local build tag** (see `Dockerfile.solum-sidecar`), not a published GHCR image. Set `SOLUM_IMAGE` or Helm `solum.image` to a real registry tag for pilots. |

`./scripts/bump-ferrum.sh` rewrites the git pin **and** both image pin files to `ghcr.io/synapticfour/ferrum:<sha>`.

## Review

Include this file in the monthly hygiene pass ([MONTHLY-DEPENDENCY-HYGIENE](https://github.com/SynapticFour/synapticfour-infra/blob/main/docs/MONTHLY-DEPENDENCY-HYGIENE.md)).
