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

Ferrum images and binaries are **placeholders** until wired to real Ferrum releases.
