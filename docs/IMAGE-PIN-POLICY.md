# Image pin policy

**Status:** 2026-08-12 · org level-up **C10**
**Repo:** Ferrum-Lab-Kit

## Policy

| Context | Rule |
|---------|------|
| **Pilot / production profiles** | Prefer **immutable tags** (SemVer / `vX.Y.Z`) or **digest pins** (`image@sha256:…`). Do not rely on floating `:latest` for customer-held pilots. |
| **Local demo / bring-your-own** | `:latest` may appear as a developer default; document that pilots must override. |
| **Third-party infra** (Traefik, Postgres, …) | Pin minor/major versions (e.g. `traefik:v3.3`); bump deliberately in a PR with notes. |
| **ga4gh-infra siblings** | Use version env vars (`MOCK_IDP_VERSION`, etc.) already present in compose; keep defaults aligned with published infra tags. |

## Current known floaters (to tighten over time)

Helm `values.yaml` and several compose fragments still default Synaptic Four service images to `:latest` / `lab-kit`. For H1/H2 pilot packs, operators should set explicit tags via Lab Kit profile / env (see Showcase `PINNED_VERSIONS.txt`).

## Review

Include this file in the monthly hygiene pass ([MONTHLY-DEPENDENCY-HYGIENE](https://github.com/SynapticFour/synapticfour-infra/blob/main/docs/MONTHLY-DEPENDENCY-HYGIENE.md)).
