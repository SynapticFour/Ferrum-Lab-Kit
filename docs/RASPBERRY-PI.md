# Raspberry Pi field edge

**Ferrum** (and optionally **Solum**) is marketed for resource-constrained field labs. Lab Kit is the **portable on-ramp** for that story: Beacon + DRS on a Pi, SQLite + local disk, no Postgres/MinIO.

Public Ferrum overview: [synapticfour.com/en/ferrum-field](https://synapticfour.com/en/ferrum-field) · upstream [AFRICA-DEPLOYMENT.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/AFRICA-DEPLOYMENT.md).

## What we promise (docs map)

| Promise | Where it lives | Lab Kit delivers |
|---------|----------------|------------------|
| Pi 5, 8 GB, 64-bit, USB SSD or NVMe | Ferrum ADR-026, Lab Kit DEPLOYMENT-TARGETS | ARM64 image. Installer refuses other boards |
| Minimal surfaces (Beacon + DRS) | `field-edge` profile | `:<sha>-edge` image + `FERRUM_SERVICES__ENABLE_*` |
| Offline-capable / SQLite | Ferrum Edge mode + Lab Kit `edge.yml` | Compose overlay + data dir |
| Optional auth plane | ga4gh-infra co-deploy | `--with-ga4gh-infra` / `field-edge+infra` |
| Optional consent on Pi | Solum H4 (Track A only) | `--with-solum` / `field-edge+solum` |
| USB SSD for objects | Ferrum AFRICA-DEPLOYMENT | Documented; point `FERRUM_DATA_DIR` at mount |

**Not on the Pi:** Solum Track B / EHRbase, full WES/TES stacks — those stay on a hub ([Solum H4-OFFLINE-SYNC-POLICY](https://github.com/SynapticFour/Solum/blob/main/docs/H4-OFFLINE-SYNC-POLICY.md), Showcase H4 hub/Pi architecture).

## Two ways to install

### A. Generate a portable kit (recommended for kits / USB)

On a laptop (any arch) with Lab Kit built:

```bash
# Minimal Ferrum on Pi
lab-kit generate raspberry-pi --output ./pi-kit
# alias: lab-kit generate pi

# Ferrum + Solum consent companion (same board)
lab-kit generate pi --output ./pi-kit --with-solum

# Ferrum + ga4gh-infra
lab-kit generate pi --profile field-edge+infra --output ./pi-kit

# All companions
lab-kit generate pi --profile field-edge+infra+solum --output ./pi-kit
```

Copy `./pi-kit` to the Pi (USB / `scp -r`), then **on the Pi**:

```bash
cd pi-kit
./install-on-pi.sh
```

Kit contents: `lab-kit.toml`, `docker-compose.yml`, `.env` (ARM64 image), `README.md`, `install-on-pi.sh`, and `deploy/docker-compose/` fragments (infra/Solum when enabled).

### B. Install directly on the Pi

```bash
curl -fsSL https://raw.githubusercontent.com/SynapticFour/Ferrum-Lab-Kit/main/install-edge.sh | bash
# or from a clone:
./install-edge.sh
./install-edge.sh --with-solum
./install-edge.sh --with-infra --with-solum
```

Same stack as the kit; uses `field-edge*` profiles and the SHA-pinned ARM64 image from `config/ci/ferrum-image-arm64.txt` (override with `FERRUM_IMAGE`).

```bash
make up                 # first run → install-edge.sh
make up-with-solum
```

## Hardware

One board for Ferrum alone and for Ferrum with Solum Track A and ga4gh-infra.

| Item | Requirement |
|------|-------------|
| Board | Raspberry Pi 5. 16 GB RAM is the same board with more memory |
| RAM | 8 GB. `install-on-pi.sh` refuses `MemTotal` under 7000 MB |
| OS | Raspberry Pi OS 64-bit or Ubuntu 24.04 ARM64 |
| Data disk | USB SSD or NVMe (M.2 HAT). The installer refuses a data directory on `mmcblk` |
| Power | Official 27 W USB-C supply (5 V / 5 A) and the Active Cooler |

Ferrum’s process cap stays **3072 MB** (container limit 3 GB). That share leaves room on the 8 GB board for the OS, Docker, Solum Track A, and ga4gh-infra. It is not a measured combined-load result.

## Placement next to hardware already in the field

The Pi kit is the only edge installer. It does not install onto the machines that already sequence or analyse.

| Machine | What it is | What this kit does with it |
|---------|------------|----------------------------|
| MinION Mk1C | Oxford Nanopore’s integrated sequencer: 8 GB RAM, 1 TB SSD, embedded GPU, MinKNOW, up to 60 W | Leave MinKNOW on the Mk1C. Point `ferrum ingest watch` at the FASTQ or raw directory it wrote, copied onto the Pi’s SSD |
| MinION Mk1B / Mk1D host | ONT requires Intel, AMD, or Apple, 16 GB RAM or more, and an NVIDIA GPU for live basecalling. ARM hosts are outside that requirement | Same file handoff. The Pi is not that host |
| Illumina MiSeq and an institute server | Africa PGI handovers (for example Cameroon, 2023: MiSeq, Mk1C, analysis server). H3ABioNet nodes run Dell PowerEdge or HP ProLiant with 128 GB RAM and more | Hub path: Lab Kit `institute` / Compose / Helm on that server. The Pi syncs to the hub |

No installer profile is generated for those machines. A profile would mean “install our stack on that box.” The Mk1C already runs MinKNOW. The rack server already has an institute install path.

## Verify

```bash
curl -fsS http://127.0.0.1:8080/health
curl -fsS http://127.0.0.1:8080/ga4gh/beacon/v2/info
curl -fsS http://127.0.0.1:8080/ga4gh/drs/v1/service-info
```

## Related

- [DEPLOYMENT-TARGETS.md](DEPLOYMENT-TARGETS.md#field-edge) — field-edge profiles
- [SOLUM-CO-DEPLOY.md](SOLUM-CO-DEPLOY.md) — Solum companion boundaries
- [FERRUM-INTEGRATION.md](FERRUM-INTEGRATION.md) — monolith image + ENABLE flags
- Demo hardware script (standalone): [Ferrum-GA4GH-Demo install-ferrum-edge.sh](https://github.com/SynapticFour/Ferrum-GA4GH-Demo/blob/main/demo/scenarios/raspberry-pi/install-ferrum-edge.sh)
