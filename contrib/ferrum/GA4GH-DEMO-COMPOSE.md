# GA4GH demo: Compose overlay (WES + TES Docker)

This file documents the **`demo/docker-compose.ga4gh.yml`** merge overlay for local **WES → TES (Docker)** workflows with **host-visible work directories**, **Docker socket**, optional **Crypt4GH** keys, and **`host.docker.internal`** for nested workflow engines.

**Baseline stack:** `deploy/docker-compose.yml` (Postgres, MinIO, Keycloak, gateway, UI).

**Run (example):**

```bash
export FERRUM_WES_WORK_HOST=/abs/path/wes-runs
export FERRUM_TES_WORK_HOST=/abs/path/tes-jobs
mkdir -p "$FERRUM_WES_WORK_HOST" "$FERRUM_TES_WORK_HOST"

docker compose -p ferrum-ga4gh \
  -f deploy/docker-compose.yml \
  -f demo/docker-compose.ga4gh.yml \
  up -d --build
```

Use **`-p ferrum-ga4gh`** so the default user-defined network is predictably **`ferrum-ga4gh_default`** (matches `FERRUM_TES_DOCKER_NETWORK` in the overlay).

---

## Environment variables (checklist)

### Host-only (Compose substitution — **not** passed to the container)

| Variable | Purpose |
|----------|---------|
| `FERRUM_WES_WORK_HOST` | **Absolute** host path bind-mounted to **`/wes-runs`** in the gateway container. |
| `FERRUM_TES_WORK_HOST` | **Absolute** host path bind-mounted to **`/tes-jobs`** (TES executor scratch). |
| `FERRUM_CRYPT4GH_KEY_HOST_DIR` | Optional. Host directory with node keypair `{id}.pub` / `{id}.sec` (default bind: `./var/ferrum-crypt4gh-keys` → container path below). |

**Spelling:** always **`FERRUM_…`** (two `R`s). A historical typo **`FERUM_WES_WORK_HOST`** is invalid — do not use in docs or `.env`.

### Inside `ferrum-gateway` (read by Rust)

| Variable | Typical value (this overlay) | Notes |
|----------|------------------------------|--------|
| `FERRUM_WES_WORK_DIR` | `/wes-runs` | Must match the **in-container** mount target for `FERRUM_WES_WORK_HOST`. |
| `FERRUM_WES_TES_URL` | `http://ferrum-gateway:8080/ga4gh/tes/v1` | In-compose TES base URL. |
| `FERRUM_TES_BACKEND` | `docker` | Selects Docker TES executor (gateway links `ferrum-tes` with `docker` feature). |
| `FERRUM_TES_WORK_DIR` | `/tes-jobs` | Executor scratch; must match mount target for `FERRUM_TES_WORK_HOST`. |
| `FERRUM_TES_DOCKER_NETWORK` | `ferrum-ga4gh_default` | Docker **network mode** name for TES task containers (same Compose project network as services). |
| `FERRUM_TES_EXTRA_HOSTS` | `host.docker.internal:host-gateway` | Propagated to TES-created containers; helps nested engines fetch workflow descriptors from the host. |
| `FERRUM_PUBLIC_BASE_URL` | e.g. `http://localhost:8080` | DRS/htsget ticket URLs — set so clients and nested tasks can reach the gateway. |
| `FERRUM_ENCRYPTION__CRYPT4GH_KEY_DIR` | `/etc/ferrum/crypt4gh-keys` | Optional; required for **encrypt=true** ingest and Crypt4GH stream behaviour (see [CRYPT4GH.md](CRYPT4GH.md)). |

---

## Mounts

| Mount | Why |
|-------|-----|
| `FERRUM_*_WORK_HOST` → `/wes-runs`, `/tes-jobs` | **Symmetric path contract** for WES/TES and nested `docker run` (see [TES-DOCKER-BACKEND.md](TES-DOCKER-BACKEND.md)). |
| `/var/run/docker.sock` | TES Docker executor talks to the host daemon. |
| Crypt4GH host dir → `/etc/ferrum/crypt4gh-keys` | Node keys for at-rest encryption / stream decrypt. |
| `extra_hosts: host.docker.internal:host-gateway` on **gateway** | Gateway process can resolve host; TES tasks get **`FERRUM_TES_EXTRA_HOSTS`** from the same environment when tasks are created. |

**Optional static `docker` CLI inside task images:** not set in Compose — if an engine shell-outs to `docker`, either install the CLI in the **workflow image** or bind-mount a binary (site-specific; see [TES-DOCKER-BACKEND.md](TES-DOCKER-BACKEND.md) § *docker.sock vs docker CLI*).

---

## Crypt4GH (optional parity with encrypted-at-rest demos)

1. Create a key directory on the host (see [CRYPT4GH.md](CRYPT4GH.md), [INGEST-LAB-KIT.md](INGEST-LAB-KIT.md)).
2. Set `FERRUM_CRYPT4GH_KEY_HOST_DIR` or use the default `./var/ferrum-crypt4gh-keys` and place `{crypt4gh_master_key_id}.pub` / `.sec` there.
3. Use **`POST /api/v1/ingest/upload`** with `encrypt=true` or equivalent DRS ingest flags as documented upstream.

---

## Related docs

- [TES-DOCKER-BACKEND.md](TES-DOCKER-BACKEND.md) — nested containers, bind mounts, `docker.sock`  
- [GA4GH.md](GA4GH.md) — WES/TES/DRS paths  
- [WORKFLOWS.md](WORKFLOWS.md) — engine submission notes  

**Ferrum Lab Kit** (deployment / integration layer): [github.com/SynapticFour/Ferrum-Lab-Kit](https://github.com/SynapticFour/Ferrum-Lab-Kit)

---

*[← Documentation index](README.md)*
