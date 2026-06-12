# Deployment targets

Lab Kit targets three environments equally (see `lab-kit generate …`).

## 1. Docker Compose (primary)

- **Use case:** single server, laptops, CI, demos.
- **Flow:** `lab-kit generate compose` → `docker compose -f generated/docker-compose.yml up`.
- **Fragments:** `deploy/docker-compose/docker-compose.*.yml` merged with `docker-compose.base.yml`.
- **Platforms:** Ubuntu 22.04/24.04, macOS (Apple Silicon), x86_64 Linux.

## 2. Kubernetes (Helm)

- **Use case:** institutional clusters, shared operations.
- **Flow:** `lab-kit generate helm` produces a values overlay; combine with chart under `deploy/helm/`.
- **Target:** Kubernetes 1.27+; all services **disabled** in default `values.yaml`.

## 3. HPC / SLURM + systemd

- **Use case:** German and DACH university HPC (login node + `sbatch`).
- **Flow:** `lab-kit generate systemd` → install units (e.g. under `/etc/systemd/system/`).
- **Gateway:** `deploy/slurm/ferrum-gateway.service` documents the **ferrum-slurm-proxy** pattern (WES/TES → SLURM).
- **Remote login node:** Ferrum integrations can use `SlurmSshComputeBackend` in `lab-kit-adapters` to run `sbatch`/`squeue` over **SSH** (key or agent; `BatchMode=yes`). Handy when the gateway runs on a VM/container without a local SLURM client.

Ferrum images and binaries are **placeholders** until wired to real Ferrum releases.

## 4. Field / Edge Deployment {#field-edge}

Minimal single-node GA4GH stack for **resource-constrained** environments: field labs, satellite offices, and offline-capable sites.

### Hardware

| Target | RAM | Notes |
|--------|-----|-------|
| **Raspberry Pi 5** (recommended) | 4–8 GB | ARM64; microSD or USB-SSD for `/data` |
| **Raspberry Pi 4** (minimum) | 4 GB | Usable with `field-edge` profile defaults |
| **Laptop** (Ubuntu 22.04/24.04) | 8–16 GB | x86_64 or ARM64 |

### Enabled services

The **`field-edge`** profile enables **Beacon v2** and **DRS** only (minimal footprint). **WES**, **TES**, **TRS**, and **htsget** are disabled by default and can be re-enabled in `lab-kit init` if local compute and bandwidth allow.

**Backend:** SQLite metadata + local filesystem object store — **no PostgreSQL or MinIO** required.

**Auth:** Local Passport validation (offline-capable). Switch to LS Login when internet is available.

### Quick install

```bash
curl -fsSL https://raw.githubusercontent.com/SynapticFour/Ferrum-Lab-Kit/main/install-edge.sh | bash
# or from a clone:
./install-edge.sh
```

The installer:

1. Installs Docker (if missing) via apt
2. Installs the `lab-kit` CLI (release binary, bundled copy, or builds from source)
3. Runs `lab-kit init --profile field-edge --non-interactive`
4. Generates `docker-compose.yml` with the `edge.yml` overlay
5. Starts the stack and verifies Beacon v2 on port **8080**

Manual path:

```bash
cp config/profiles/field-edge.toml lab-kit.toml   # or: lab-kit init --profile field-edge
lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose --output docker-compose.yml
docker compose up -d
```

### Nanopore MinION in the field

For Oxford Nanopore **MinION** sequencing in low-infrastructure settings, wire the sequencer output directory into Ferrum ingest and DRS object paths. See Ferrum Africa documentation (when available in upstream) for MinION-specific ingest flows; the edge profile’s `FERRUM_AFRICA__*` environment variables activate those features when implemented.

### Federation with a national node

When intermittent connectivity returns, register this edge node with your **GDI / ELIXIR national node** using Beacon Network federation. See [ELIXIR AAI](ELIXIR-AAI.md) for LS Login setup and your national node operator for federation endpoints.

### Power and connectivity

- **Intermittent internet:** opportunistic sync defaults to `0 2 * * *` (2 AM) when bandwidth is available (`[network].bandwidth_adaptive`).
- **Solar / battery:** the edge overlay sets power-monitor thresholds (`FERRUM_AFRICA__LOW_POWER_THRESHOLD`, `FERRUM_AFRICA__EMERGENCY_THRESHOLD`). These are **ignored by stock Ferrum** until Africa Cursor Prompts land upstream.
- **Memory:** default `max_memory_mb = 3072` leaves headroom on 4 GB Pi; increase via `lab-kit init` → “Expected RAM (GB)?”.
