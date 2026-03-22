# Ferrum GA4GH demo overlay (WES + TES Docker)

Lab Kit **does not** replace Ferrum’s `deploy/docker-compose.yml`. For a **single, documented merge overlay** (WES workdir on a **host-visible** path, **TES via Docker**, **`docker.sock`**, optional **Crypt4GH** keys, **`host.docker.internal`** / `FERRUM_TES_EXTRA_HOSTS`), the canonical spec lives in the **Ferrum** repository:

- **`demo/docker-compose.ga4gh.yml`** — merge with `deploy/docker-compose.yml`
- **`docs/GA4GH-DEMO-COMPOSE.md`** — env checklist and run commands

## Mirrors & patch (apply in a Ferrum clone)

In this repo:

| Path | Use |
|------|-----|
| [contrib/ferrum/demo-docker-compose.ga4gh.yml](../contrib/ferrum/demo-docker-compose.ga4gh.yml) | Copy to Ferrum `demo/docker-compose.ga4gh.yml` |
| [contrib/ferrum/GA4GH-DEMO-COMPOSE.md](../contrib/ferrum/GA4GH-DEMO-COMPOSE.md) | Copy to Ferrum `docs/GA4GH-DEMO-COMPOSE.md` |
| [contrib/ferrum/0001-ga4gh-demo-compose-wes-tes-docker.patch](../contrib/ferrum/0001-ga4gh-demo-compose-wes-tes-docker.patch) | **`git apply`** — includes Compose + docs + gateway **`FERRUM_TES_*`** wiring + Docker TES executor fixes |

See [contrib/ferrum/README.md](../contrib/ferrum/README.md).

## Naming: host vs container (avoid `FERUM_*` typo)

| Where | Variable | Role |
|-------|----------|------|
| **Host / `.env` for Compose only** | `FERRUM_WES_WORK_HOST`, `FERRUM_TES_WORK_HOST` | Left-hand side of **bind mounts** (absolute paths). **Not** read by Rust. |
| **Inside `ferrum-gateway`** | `FERRUM_WES_WORK_DIR`, `FERRUM_TES_WORK_DIR` | Paths the **binary** uses — must equal the **container mount targets** (`/wes-runs`, `/tes-jobs` in the overlay). |

Rust already used **`FERRUM_WES_WORK_DIR`** (not `…_WORK_HOST`). Use a **distinct** `*_WORK_HOST` name on the host so Compose substitution is not confused with in-container config.

**Invalid:** `FERUM_WES_WORK_HOST` (missing `R`) — do not use.

## Lab Kit Compose fragments

Generated files under `deploy/docker-compose/*.yml` follow **Lab Kit** service selection; they are **not** identical to Ferrum’s full stack. When documenting env vars for operators, **align names** with the Ferrum GA4GH overlay (`FERRUM_WES_WORK_HOST`, `FERRUM_TES_*`, `FERRUM_ENCRYPTION__CRYPT4GH_KEY_DIR`) where the same concept applies.

## Further reading

- [Ferrum `docs/TES-DOCKER-BACKEND.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/TES-DOCKER-BACKEND.md)  
- [GA4GH workflow primer](GA4GH-WORKFLOW-PRIMER.md) · [Operations checklist](OPERATIONS-CHECKLIST.md)  
- [Ferrum integration](FERRUM-INTEGRATION.md)

---

*[← Documentation index](README.md)*
