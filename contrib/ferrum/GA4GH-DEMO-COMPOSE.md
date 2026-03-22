# GA4GH demo: Compose overlay (WES + TES Docker)

Merge **`demo/docker-compose.ga4gh.yml`** with **`deploy/docker-compose.yml`** for a local stack where **WES** uses a **host-visible work directory**, **TES** runs tasks on the **Docker** daemon (requires gateway image built with **`tes-docker`** — the overlay sets build arg **`FERRUM_GATEWAY_FEATURES`**).

**Run:**

```bash
export FERRUM_WES_WORK_HOST=/abs/path/wes-runs
export FERRUM_TES_WORK_HOST=/abs/path/tes-jobs
mkdir -p "$FERRUM_WES_WORK_HOST" "$FERRUM_TES_WORK_HOST"

docker compose -p ferrum-ga4gh \
  -f deploy/docker-compose.yml \
  -f demo/docker-compose.ga4gh.yml \
  up -d --build
```

Use **`-p ferrum-ga4gh`** so the default network is **`ferrum-ga4gh_default`**, matching **`FERRUM_TES_DOCKER_NETWORK_MODE`** in the overlay.

---

## Host vs container variables

| Where | Variables | Role |
|-------|-----------|------|
| **Host / Compose `.env`** | `FERRUM_WES_WORK_HOST`, `FERRUM_TES_WORK_HOST` | Left side of **bind mounts** only. **Not** read by Rust. |
| **Inside `ferrum-gateway`** | `FERRUM_WES_WORK_DIR`, `FERRUM_TES_WORK_DIR` | Paths the binary uses — must match mount targets (`/wes-runs`, `/tes-jobs`). |

**Spelling:** always **`FERRUM_`** (two `R`s). **`FERUM_WES_WORK_HOST`** is a common typo — invalid.

---

## Environment (this overlay)

| Variable | Typical value | Notes |
|----------|---------------|--------|
| `FERRUM_TES_BACKEND` | `docker` | Requires **`tes-docker`** gateway build (see [TES-DOCKER-BACKEND.md](TES-DOCKER-BACKEND.md)). |
| `FERRUM_WES_TES_WORK_HOST_PREFIX` | same as `FERRUM_WES_WORK_HOST` | Passed from host via Compose; WES adds per-run TES volume binds for nested `docker run`. |
| `FERRUM_TES_DOCKER_NETWORK_MODE` | `ferrum-ga4gh_default` | Docker `NetworkMode` for TES task containers. |
| `FERRUM_TES_DOCKER_EXTRA_HOSTS` | `host.docker.internal:host-gateway` | Comma-separated; see TES-DOCKER-BACKEND. |
| `FERRUM_TES_DOCKER_MOUNT_SOCKET` | `1` | Truthy → bind docker.sock **into each TES task** (nested engines). |
| `FERRUM_ENCRYPTION__CRYPT4GH_KEY_DIR` | `/etc/ferrum/crypt4gh-keys` | Optional; for encrypt-at-rest / stream (see [CRYPT4GH.md](CRYPT4GH.md)). |

Full table of **`FERRUM_TES_DOCKER_*`** knobs: [TES-DOCKER-BACKEND.md — TES Docker executor](TES-DOCKER-BACKEND.md#tes-docker-executor-optional-environment-site-specific).

**Default demo / CI:** `deploy/docker-compose.yml` sets **`FERRUM_TES_BACKEND=${FERRUM_TES_BACKEND:-noop}`** so **HelixTest** and checksum alignment keep **noop** unless you override.

---

## Optional: static `docker` CLI in tasks

See **`FERRUM_TES_DOCKER_CLI_HOST_PATH`** / **`FERRUM_TES_DOCKER_CLI_CONTAINER_PATH`** in [TES-DOCKER-BACKEND.md](TES-DOCKER-BACKEND.md).

---

## Related

- [TES-DOCKER-BACKEND.md](TES-DOCKER-BACKEND.md)  
- [GA4GH.md](GA4GH.md) · [WORKFLOWS.md](WORKFLOWS.md)  
- [Ferrum Lab Kit](https://github.com/SynapticFour/Ferrum-Lab-Kit)

---

*[← Documentation index](README.md)*
