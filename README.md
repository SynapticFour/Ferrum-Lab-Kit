# Ferrum Lab Kit

**Ferrum Lab Kit** is the **on-ramp** to [Ferrum](https://github.com/SynapticFour/Ferrum): a **deployment and integration layer** for small and mid-size research labs, **ELIXIR node candidates**, **GHGA** data submitters, and **GDI** national-node participants who need **selective GA4GH-aligned services** without running the full Ferrum platform. It is a **separate repository** — not a fork — and **does not duplicate** Ferrum’s GA4GH implementations; it configures and ships them against **your** storage, scheduler, and identity stack.

> **Legal notice:** This repository documents technical capabilities and operating guidance. It is not legal advice and does not by itself provide regulatory certification or compliance guarantees. Compliance outcomes depend on operator configuration, contracts, and organisational controls.

## SynapticFour GA4GH stack

Lab Kit is the **deployment on-ramp**. See **[docs/ECOSYSTEM.md](docs/ECOSYSTEM.md)** for Ferrum, ga4gh-infra, Demo, and HelixTest.

## Install CLI (optional)

From a clone, build or install the `lab-kit` binary (needs [Rust](https://rustup.rs)):

```bash
./install.sh              # release build → target/release/lab-kit
./install.sh --install    # also cargo install (default: ~/.cargo/bin)
./install.sh --install --prefix "$HOME/.local"   # → ~/.local/bin
```

## Shortest path: Beacon v2 + DRS (~5 commands)

```bash
git clone https://github.com/SynapticFour/Ferrum-Lab-Kit.git && cd Ferrum-Lab-Kit
cp .env.example .env
lab-kit init --profile field-edge --non-interactive   # or: cargo run -p lab-kit-selector -- …
lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose -o docker-compose.yml
docker compose -f docker-compose.yml up -d
# Gateway: http://127.0.0.1:8080/health
# Beacon:  http://127.0.0.1:8080/ga4gh/beacon/v2/info
```

Or one-shot: `./install-edge.sh` / `make up`.

### Raspberry Pi kit

```bash
lab-kit generate raspberry-pi --output ./pi-kit          # or: make pi-kit
# On the Pi: cd pi-kit && ./install-on-pi.sh
lab-kit generate pi --with-solum --ram-gb 8 -o ./pi-kit  # Ferrum + Solum
```

Full guide: [docs/RASPBERRY-PI.md](docs/RASPBERRY-PI.md).

### How selective deploy works

`lab-kit.toml` / profiles choose GA4GH surfaces. Generators emit a **single monolith** service (`ferrum-gateway` → `ghcr.io/synapticfour/ferrum`) and set:

`FERRUM_SERVICES__ENABLE_BEACON|DRS|HTSGET|WES|TES|TRS`

| Image | Notes |
|-------|--------|
| `ghcr.io/synapticfour/ferrum` | Gateway / platform (`latest`, `latest-arm64`, SHA, `v*`) |
| `ghcr.io/synapticfour/ferrum-ui` | Optional UI (not required by Lab Kit) |

Override with `FERRUM_IMAGE` in `.env`. Details: [docs/FERRUM-INTEGRATION.md](docs/FERRUM-INTEGRATION.md).

## Co-deploy with ga4gh-infra

Ferrum and [ga4gh-infra](https://github.com/SynapticFour/ga4gh-infra) belong together for Passport broker + service registry:

```bash
./install-edge.sh --with-infra
# or: make up-with-infra
# Profiles: field-edge+infra.toml, institute.toml
```

## Optional Solum companion

Purpose-bound consent checks (Ferrum polls Solum) without moving clinical product ownership into Lab Kit:

```bash
./install-edge.sh --with-solum
# or: make up-with-solum / make up-with-infra-solum
```

See [docs/SOLUM-CO-DEPLOY.md](docs/SOLUM-CO-DEPLOY.md).

### Local lifecycle (Make)

| Goal | Standalone | + ga4gh-infra | + Solum |
|------|------------|---------------|---------|
| Start | `make up` | `make up-with-infra` | `make up-with-solum` |
| Both companions | | `make up-with-infra-solum` | |
| Stop (keep data) | `make down` | same | same |
| Remove volumes | `make destroy` | same | same |

## Service selection (what to enable)

| GA4GH surface | What it enables (examples) |
|---------------|----------------------------|
| **Beacon v2** | ELIXIR Beacon Network, public/registered/controlled cohort discovery |
| **DRS** | Stable data object IDs over S3/POSIX |
| **htsget** | Efficient genomic data streaming |
| **WES / TES** | Portable workflows and task execution on SLURM/K8s |
| **TRS** | Tool/workflow registry (e.g. nf-core) |

Details: [docs/GA4GH-STANDARDS.md](docs/GA4GH-STANDARDS.md).

## Who this is for

- University and institute labs (**DE / AT / CH** and beyond) on **SLURM** or single servers.
- **ELIXIR node** candidates needing a documented, conformance-tested subset.
- **GDI** national node and **rare disease** consortia attaching evidence to applications.
- **NFDI** and related research-data initiatives composing standards-based services.
- **Field labs** in resource-constrained settings (Africa, remote sites) running Nanopore sequencing on Raspberry Pi or laptops — see [Field/Edge deployment](docs/DEPLOYMENT-TARGETS.md#field-edge).

## Open core

**GA4GH deployment and LS Login integration** are open under **BUSL-1.1** (see [LICENSE](LICENSE)) for permitted non-commercial research use. **Conformance PDF reports** and enterprise federation tooling are **commercial** offerings — PDF output requires a well-formed **`FERRUM_LAB_KIT_LICENSE_KEY`** and **`lab-kit license activate`**; **JSON reports and protocol stacks are not license-gated.** See [docs/BUSINESS-MODEL.md](docs/BUSINESS-MODEL.md).

## CLI (`lab-kit`)

| Command | Purpose |
|---------|---------|
| `lab-kit init` | Interactive wizard → `lab-kit.toml` |
| `lab-kit generate compose` | Merge compose fragments (monolith + optional infra/Solum) |
| `lab-kit generate compose --with-ga4gh-infra` | Force ga4gh-infra co-deploy |
| `lab-kit generate compose --with-solum` | Force Solum sidecar companion |
| `lab-kit generate raspberry-pi` / `pi` | Portable Pi field kit (`pi-kit/` + `install-on-pi.sh`) |
| `lab-kit generate helm` | Emit values overlay (`gateway.enable.*`) |
| `lab-kit generate systemd` | Emit `ferrum-gateway.service` (+ optional Solum) |
| `lab-kit status` | Health table for enabled services |
| `lab-kit conformance run` | Invoke external **HelixTest** CLI |
| `lab-kit conformance report` | JSON (+ optional licensed PDF) |
| `lab-kit ferrum check` | Confirms Git-pinned `ferrum-core` from [Ferrum](https://github.com/SynapticFour/Ferrum) resolves |
| `lab-kit ingest …` | HTTP client for Ferrum **`/api/v1/ingest/*`** — see [Ferrum `docs/INGEST-LAB-KIT.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/INGEST-LAB-KIT.md) |
| `lab-kit mii sync-manifest` | Optional wrapper for `ferrum mii sync-manifest` |
| `lab-kit mii validate` | Optional wrapper for `ferrum mii validate` |

MII helpers are intentionally optional. Lab Kit remains GA4GH-centric; MII handling is delegated to upstream Ferrum MII Connect.

## Documentation

- **[Documentation index](docs/README.md)** — all guides and examples
- [GA4GH workflow primer](docs/GA4GH-WORKFLOW-PRIMER.md) — TRS/WES/TES flow, DRS, engines, nested Docker, `amd64`/`arm64`
- [Operations checklist](docs/OPERATIONS-CHECKLIST.md) — env vars, Docker, networking, naming
- [Solum co-deploy](docs/SOLUM-CO-DEPLOY.md) — optional consent companion
- [Raspberry Pi](docs/RASPBERRY-PI.md) — field kit + on-device install
- [Ferrum GA4GH demo overlay](docs/FERRUM-GA4GH-DEMO-OVERLAY.md) — WES/TES Docker Compose merge + `contrib/ferrum/` patch

Also: [Architecture](docs/ARCHITECTURE.md) · [Ferrum integration](docs/FERRUM-INTEGRATION.md) · [Deployment targets](docs/DEPLOYMENT-TARGETS.md) · [ELIXIR AAI](docs/ELIXIR-AAI.md) · [Bring your own](docs/BRING-YOUR-OWN.md) · [Conformance](docs/CONFORMANCE.md) · [Business model](docs/BUSINESS-MODEL.md)

## Need the full platform?

Go to **[github.com/SynapticFour/Ferrum](https://github.com/SynapticFour/Ferrum)** for the complete sovereign stack.

## German README

See [README.de.md](README.de.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
