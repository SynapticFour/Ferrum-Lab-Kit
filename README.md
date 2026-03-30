# Ferrum Lab Kit

**Ferrum Lab Kit** is the **on-ramp** to [Ferrum](https://github.com/SynapticFour/Ferrum): a **deployment and integration layer** for small and mid-size research labs, **ELIXIR node candidates**, **GHGA** data submitters, and **GDI** national-node participants who need **selective GA4GH-aligned services** without running the full Ferrum platform. It is a **separate repository** — not a fork — and **does not duplicate** Ferrum’s GA4GH implementations; it configures and ships them against **your** storage, scheduler, and identity stack.

This repository provides technical deployment and conformance tooling. It does not constitute legal advice or a formal compliance certification for any jurisdiction.

## Install CLI (optional)

From a clone, build or install the `lab-kit` binary (needs [Rust](https://rustup.rs)):

```bash
./install.sh              # release build → target/release/lab-kit
./install.sh --install    # also cargo install (default: ~/.cargo/bin)
./install.sh --install --prefix "$HOME/.local"   # → ~/.local/bin
```

## Shortest path: Beacon v2 + ELIXIR LS Login (~5 commands)

```bash
git clone https://github.com/SynapticFour/Ferrum-Lab-Kit.git && cd Ferrum-Lab-Kit
cp config/profiles/beacon-only.toml lab-kit.toml
# Edit lab-kit.toml: set real [auth.ls-login] client_id / client_secret
cargo run -p lab-kit-selector -- generate compose --config lab-kit.toml --fragments deploy/docker-compose --output docker-compose.yml
docker compose -f docker-compose.yml up -d
# When Ferrum images are available — until then, images in compose are placeholders.
```

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

## Open core

**GA4GH deployment and LS Login integration** are open under **BUSL-1.1** (see [LICENSE](LICENSE)) for permitted non-commercial research use. **Conformance PDF reports** and enterprise federation tooling are **commercial** offerings — PDF output checks **`FERRUM_LAB_KIT_LICENSE_KEY`**; **JSON reports and protocol stacks are not license-gated.** See [docs/BUSINESS-MODEL.md](docs/BUSINESS-MODEL.md).

## CLI (`lab-kit`)

| Command | Purpose |
|---------|---------|
| `lab-kit init` | Interactive wizard → `lab-kit.toml` |
| `lab-kit generate compose` | Merge `deploy/docker-compose/*.yml` |
| `lab-kit generate helm` | Emit values overlay |
| `lab-kit generate systemd` | Emit `ferrum-*.service` stubs |
| `lab-kit status` | Health table for enabled services |
| `lab-kit conformance run` | Invoke external **HelixTest** CLI |
| `lab-kit conformance report` | JSON (+ optional licensed PDF) |
| `lab-kit ferrum check` | Confirms Git-pinned `ferrum-core` from [Ferrum](https://github.com/SynapticFour/Ferrum) resolves |
| `lab-kit ingest …` | HTTP client for Ferrum **`/api/v1/ingest/*`** (register, upload, job status) — see [Ferrum `docs/INGEST-LAB-KIT.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/INGEST-LAB-KIT.md) |

## Documentation

- **[Documentation index](docs/README.md)** — all guides and examples  
- [GA4GH workflow primer](docs/GA4GH-WORKFLOW-PRIMER.md) — TRS/WES/TES flow, DRS, engines, nested Docker, `amd64`/`arm64`  
- [Operations checklist](docs/OPERATIONS-CHECKLIST.md) — env vars, Docker, networking, naming  
- [Ferrum GA4GH demo overlay](docs/FERRUM-GA4GH-DEMO-OVERLAY.md) — WES/TES Docker Compose merge + `contrib/ferrum/` patch  

Also: [Architecture](docs/ARCHITECTURE.md) · [Ferrum integration](docs/FERRUM-INTEGRATION.md) · [Deployment targets](docs/DEPLOYMENT-TARGETS.md) · [ELIXIR AAI](docs/ELIXIR-AAI.md) · [Bring your own](docs/BRING-YOUR-OWN.md) · [Conformance](docs/CONFORMANCE.md) · [Business model](docs/BUSINESS-MODEL.md)

## Need the full platform?

Go to **[github.com/SynapticFour/Ferrum](https://github.com/SynapticFour/Ferrum)** for the complete sovereign stack.

## German README

See [README.de.md](README.de.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
