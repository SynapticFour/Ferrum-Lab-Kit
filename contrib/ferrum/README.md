# Upstream Ferrum snippets & patches

This directory holds **mirrors** and a **unified diff** for changes that belong in **[SynapticFour/Ferrum](https://github.com/SynapticFour/Ferrum)** (not duplicated in Lab Kit product code).

## GA4GH demo Compose overlay (WES + TES Docker)

| File | Purpose |
|------|---------|
| [`demo-docker-compose.ga4gh.yml`](demo-docker-compose.ga4gh.yml) | Copy to Ferrum **`demo/docker-compose.ga4gh.yml`** (or apply patch). |
| [`GA4GH-DEMO-COMPOSE.md`](GA4GH-DEMO-COMPOSE.md) | Copy to Ferrum **`docs/GA4GH-DEMO-COMPOSE.md`**. |
| [`0001-ga4gh-demo-compose-wes-tes-docker.patch`](0001-ga4gh-demo-compose-wes-tes-docker.patch) | Full patch: overlay + docs + **`ferrum-gateway` TES env** + **`ferrum-tes` Docker executor** (`FERRUM_TES_DOCKER_NETWORK`, `FERRUM_TES_EXTRA_HOSTS`) + README / index links. |

**Apply in a Ferrum clone:**

```bash
cd /path/to/Ferrum
git apply /path/to/Ferrum-Lab-Kit/contrib/ferrum/0001-ga4gh-demo-compose-wes-tes-docker.patch
# resolve if paths differ; or: git am < patch if formatted as mailbox
```

**Lab Kit doc (operator-oriented):** [docs/FERRUM-GA4GH-DEMO-OVERLAY.md](../../docs/FERRUM-GA4GH-DEMO-OVERLAY.md)

**Env naming:** host-side volume sources use **`FERRUM_WES_WORK_HOST`** / **`FERRUM_TES_WORK_HOST`** (Compose only). The gateway reads **`FERRUM_WES_WORK_DIR`** / **`FERRUM_TES_WORK_DIR`** inside the container. Do **not** use the typo **`FERUM_*`**.
