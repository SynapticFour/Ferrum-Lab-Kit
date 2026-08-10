# Raspberry Pi field edge

**Ferrum** (and optionally **Solum**) is marketed for resource-constrained field labs. Lab Kit is the **portable on-ramp** for that story: Beacon + DRS on a Pi, SQLite + local disk, no Postgres/MinIO.

Public Ferrum overview: [synapticfour.com/en/ferrum-field](https://synapticfour.com/en/ferrum-field) · upstream [AFRICA-DEPLOYMENT.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/AFRICA-DEPLOYMENT.md).

## What we promise (docs map)

| Promise | Where it lives | Lab Kit delivers |
|---------|----------------|------------------|
| Pi 5 as primary edge hardware | Ferrum AFRICA-DEPLOYMENT, Lab Kit DEPLOYMENT-TARGETS | ARM64 image + RAM checks |
| Minimal surfaces (Beacon + DRS) | `field-edge` profile | `FERRUM_SERVICES__ENABLE_*` |
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

# Ferrum + Solum consent companion
lab-kit generate pi --output ./pi-kit --with-solum --ram-gb 8

# Ferrum + ga4gh-infra
lab-kit generate pi --profile field-edge+infra --output ./pi-kit --ram-gb 8

# All companions
lab-kit generate pi --profile field-edge+infra+solum --output ./pi-kit --ram-gb 8
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

Same stack as the kit; uses `field-edge*` profiles and pulls `ghcr.io/synapticfour/ferrum:latest-arm64` on aarch64.

```bash
make up                 # first run → install-edge.sh
make up-with-solum
```

## Hardware

| Target | RAM | Notes |
|--------|-----|-------|
| **Pi 5** (recommended) | 4–8 GB | Prefer USB SSD for `~/.ferrum` |
| **Pi 4** (minimum) | 4 GB | Beacon+DRS only; avoid Solum/infra on 4 GB |
| + Solum or ga4gh-infra | **8 GB** | Sidecar + broker need headroom |

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
