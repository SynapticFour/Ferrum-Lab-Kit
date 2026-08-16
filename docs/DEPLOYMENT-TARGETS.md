# Deployment targets

Lab Kit targets three environments equally (see `lab-kit generate …`).

## 1. Docker Compose (primary)

- **Use case:** single server, laptops, CI, demos.
- **Flow:** `lab-kit generate compose` → `docker compose -f docker-compose.yml up -d`.
- **Default runtime:** named Ferrum variant as `ferrum-gateway` on **8080** (`:<sha>` full, `:<sha>-edge`, `:<sha>-edge-infra`) plus `FERRUM_SERVICES__ENABLE_*` from your profile.
- **Fragments:** `docker-compose.base.yml` + `docker-compose.gateway.yml` (+ `edge.yml` / `infra.yml` / `solum.yml` / `bra.yml` as needed).
- **Platforms:** Ubuntu 22.04/24.04, macOS (Apple Silicon), x86_64 Linux, Raspberry Pi 5 (use the SHA pin in `config/ci/ferrum-image-arm64.txt`, override with `FERRUM_IMAGE`).
- **Env:** copy [`.env.example`](../.env.example) → `.env`.

## 2. Kubernetes (Helm)

- **Use case:** institutional clusters, shared operations.
- **Flow:** `lab-kit generate helm` produces a values overlay with `gateway.enable.*`; combine with chart under `deploy/helm/` (`deployment-gateway.yaml`, optional `deployment-solum.yaml`).
- **Target:** Kubernetes 1.27+; gateway **disabled** in default `values.yaml` until generate enables it.

## 3. HPC / SLURM + systemd

- **Use case:** German and DACH university HPC (login node + `sbatch`).
- **Flow:** `lab-kit generate systemd` → install `ferrum-gateway.service` (and optional `solum-sidecar.service`).
- **Gateway:** `deploy/slurm/ferrum-gateway.service` documents the **ferrum-slurm-proxy** pattern (WES/TES → SLURM).
- **Remote login node:** Ferrum integrations can use `SlurmSshComputeBackend` in `lab-kit-adapters` to run `sbatch`/`squeue` over **SSH** (key or agent; `BatchMode=yes`). Handy when the gateway runs on a VM/container without a local SLURM client.

Compose/Helm default to the **published monolith**. Legacy per-service placeholders remain behind `--legacy-per-service` — see [FERRUM-INTEGRATION.md](FERRUM-INTEGRATION.md).

## Managed single-tenant (portfolio H5)

When Synaptic Four (or a partner) **hosts** Ferrum for a customer, prefer a **dedicated** Compose/Helm install per customer (isolated project/VPC) — “hosted on-prem”, not shared multi-tenant SaaS. Align secrets, keys, and Solum profile per deployment. Portfolio contract: Showcase [ADR 0003](https://github.com/SynapticFour/SynapticFour-Showcase/blob/main/docs/adr/0003-tenant-boundaries.md) · [H5-MANAGED-SINGLE-TENANT.md](https://github.com/SynapticFour/SynapticFour-Showcase/blob/main/docs/pilots/H5-MANAGED-SINGLE-TENANT.md). Optional Solum companion: [SOLUM-CO-DEPLOY.md](SOLUM-CO-DEPLOY.md). Optional BRA companion: [BRA-CO-DEPLOY.md](BRA-CO-DEPLOY.md).

## 4. Field / Edge Deployment {#field-edge}

Minimal single-node GA4GH stack for **resource-constrained** environments: field labs, satellite offices, and offline-capable sites.

### Hardware

| Target | RAM | Notes |
|--------|-----|-------|
| **Raspberry Pi 5** (recommended) | 4–8 GB | ARM64; microSD or USB-SSD for `/data` |
| **Raspberry Pi 4** (minimum) | 4 GB | Usable with `field-edge` profile defaults |
| **Laptop** (Ubuntu 22.04/24.04) | 8–16 GB | x86_64 or ARM64 |

### Enabled services

The **`field-edge`** profile enables **Beacon v2** and **DRS** only (minimal footprint) on the monolith gateway. **WES**, **TES**, **TRS**, and **htsget** are disabled by default and can be re-enabled in `lab-kit init` if local compute and bandwidth allow.

**Companions:**

| Profile / flag | Adds |
|----------------|------|
| `field-edge+infra` / `--with-infra` | ga4gh-infra auth plane |
| `field-edge+solum` / `--with-solum` | Solum sidecar on **8787** |
| `bra-companion` / `--with-bra` | BRA workbench client (`FERRUM_DRS_URL` / `FERRUM_WES_URL`). Bring `BRA_IMAGE`. |
| `archive-submitter` | Edge + Metadata Store, no WES/TES. Reviewable archive YAML + DRS IDs — **not** EGA upload. |
| `federated-node` | Beacon+DRS+ga4gh-infra+TLS. Technical node — **not** EGA/GDI/ELIXIR membership. [FEDERATED-NODE.md](FEDERATED-NODE.md). |
| `field-edge+infra+solum` | both |

**Backend:** SQLite metadata + local filesystem object store — **no PostgreSQL or MinIO** required.

**Auth:** Local Passport validation (offline-capable). Switch to LS Login when internet is available.

### Quick install

**Portable kit** (generate on a laptop, run on the Pi):

```bash
lab-kit generate raspberry-pi --output ./pi-kit
# Optional: --with-solum / --with-ga4gh-infra / --profile field-edge+infra+solum --ram-gb 8
# Copy pi-kit/ to the Pi, then:
cd pi-kit && ./install-on-pi.sh
```

See [RASPBERRY-PI.md](RASPBERRY-PI.md).

**On-device installer:**

```bash
curl -fsSL https://raw.githubusercontent.com/SynapticFour/Ferrum-Lab-Kit/main/install-edge.sh | bash
# or from a clone:
./install-edge.sh
./install-edge.sh --with-infra
./install-edge.sh --with-solum
./install-edge.sh --with-infra --with-solum
```

The installer:

1. Installs Docker (if missing) via apt
2. Installs the `lab-kit` CLI (release binary, bundled copy, or builds from source)
3. Runs `lab-kit init --profile field-edge[…] --non-interactive`
4. Generates `docker-compose.yml` (monolith gateway + optional overlays)
5. Starts the stack and verifies the gateway / Beacon on port **8080**

Manual path:

```bash
cp .env.example .env
lab-kit init --profile field-edge --non-interactive
lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose --output docker-compose.yml
docker compose -f docker-compose.yml up -d
```

### Nanopore MinION in the field

For Oxford Nanopore **MinION** sequencing in low-infrastructure settings, wire the sequencer output directory into Ferrum ingest and DRS object paths. See Ferrum Africa documentation (when available in upstream) for MinION-specific ingest flows; the edge profile’s `FERRUM_AFRICA__*` environment variables activate those features when implemented.

### Federation with a national node

When intermittent connectivity returns, register this edge node with your **GDI / ELIXIR national node** using Beacon Network federation. See [ELIXIR AAI](ELIXIR-AAI.md) for LS Login setup and your national node operator for federation endpoints.

### Power and connectivity

- **Intermittent internet:** opportunistic sync defaults to `0 2 * * *` (2 AM) when bandwidth is available (`[network].bandwidth_adaptive`).
- **Solar / battery:** the edge overlay sets power-monitor thresholds (`FERRUM_AFRICA__LOW_POWER_THRESHOLD`, `FERRUM_AFRICA__EMERGENCY_THRESHOLD`). These are **ignored by stock Ferrum** unless the pinned image implements them.
- **Memory:** default `max_memory_mb = 3072` leaves headroom on 4 GB Pi; increase via `lab-kit init` → “Expected RAM (GB)?”.
