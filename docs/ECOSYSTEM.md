# Synaptic Four — this repo in the portfolio

Four **products**, two free **ambassadors**, Ferrum **companions**, and **proof** repos. Glue is GA4GH; Solum extends into clinical data. **Not a bundle SKU.** Canonical map: [Ferrum PORTFOLIO.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/PORTFOLIO.md).

**You are here:** [Ferrum-Lab-Kit](https://github.com/SynapticFour/Ferrum-Lab-Kit) — **comes with Ferrum**, not sold separately. Selects 1–3 GA4GH surfaces and generates Compose/Helm/systemd around the Ferrum image.

## Repositories

| Kind | Repository | Role | License |
|------|------------|------|---------|
| With Ferrum | **Ferrum-Lab-Kit** (this repo) | Subset install / on-ramp | BUSL-1.1 |
| Product | [Ferrum](https://github.com/SynapticFour/Ferrum) | GA4GH data/compute | BUSL-1.1 |
| Product | [ga4gh-infra](https://github.com/SynapticFour/ga4gh-infra) | Identity plane | Apache-2.0 |
| Ambassador | [HelixTest](https://github.com/SynapticFour/HelixTest) | Conformance CLI | Apache-2.0 |
| Proof | [Ferrum-GA4GH-Demo](https://github.com/SynapticFour/Ferrum-GA4GH-Demo) | Local `./run` smoke | Apache-2.0 |

## Ownership boundaries

| Layer | Owner | Notes |
|-------|--------|--------|
| Identity | **ga4gh-infra** | Broker, visas, DUO, ADS, service registry |
| Data/compute | **Ferrum** | DRS, WES/TES, TRS, Beacon; built-in passports in standalone mode |
| Deployment | **Ferrum-Lab-Kit** | Selective GA4GH surfaces for labs; Git-pins `ferrum-core` |
| Demo/benchmark | **Ferrum-GA4GH-Demo** | Reproducible GIAB benchmark; optional `--with-infra` |
| Conformance | **HelixTest** | Automated API and workflow tests |

Lab Kit **generates** compose/systemd/helm artefacts; Ferrum and ga4gh-infra provide the services. See [FERRUM-INTEGRATION.md](FERRUM-INTEGRATION.md).

## Default co-deploy ports

| Service | Standalone Ferrum | Co-deploy (demo / lab) |
|---------|-------------------|-------------------------|
| Ferrum gateway | 8080 | 18080 (demo) or **8080** (lab) |
| AAI broker | — | 8180 |
| Visa registry | — | 8181 |
| DUO | — | 8182 |
| Service registry | — | 8183 |
| ADS | — | 8190 |
| mock-idp | — | 9100 |

## Local lifecycle (unified commands)

Repos that run a **local Docker stack** share the same verbs:

| Verb | Meaning |
|------|---------|
| **up** | Install (if needed) and start |
| **down** | Stop containers; **keep volumes** |
| **destroy** | Stop containers and **remove volumes** |

| Repository | Deploy | Stop | Destroy | Notes |
|------------|--------|------|---------|-------|
| **ga4gh-infra** | `make up` / `just up` | `make down` | `make destroy` | Native binary: [ga4gh-infra getting-started](https://github.com/SynapticFour/ga4gh-infra/blob/main/docs/getting-started.md) |
| **Ferrum** | `make up` / `ferrum demo start` | `make down` | `make destroy` | Laptop: `ferrum demo start --offline` |
| **Ferrum-Lab-Kit** | `make up` | `make down` | `make destroy` | Co-deploy: `make up-with-infra` · Solum: `make up-with-solum` |
| **Ferrum-GA4GH-Demo** | `make up` / `./run` | `make down` | `make destroy` | Co-deploy: `make up-with-infra` |
| **HelixTest** | — | — | — | Conformance runner (needs a running target) |

**Multi-repo co-deploy** (Ferrum + ga4gh-infra):

```bash
# Benchmark path (Demo)
cd Ferrum-GA4GH-Demo && make up-with-infra
make down        # or make destroy

# Field edge path (Lab Kit)
cd Ferrum-Lab-Kit && make up-with-infra
make down        # or make destroy
```

Secondary options (always available): repo `scripts/stack-*.sh`, raw `docker compose`, and paths documented in each README.

## Quick starts

**Benchmark + co-deploy (demo):**

```bash
export FERRUM_SRC=/path/to/Ferrum
export GA4GH_INFRA_SRC=/path/to/ga4gh-infra
cd Ferrum-GA4GH-Demo && ./run --with-infra
```

**Field edge + infra (this repo):**

```bash
./install-edge.sh --with-infra
# or: lab-kit init → profile field-edge+infra → lab-kit generate compose --with-ga4gh-infra
```

**Field edge + Solum companion:**

```bash
./install-edge.sh --with-solum
# docs: docs/SOLUM-CO-DEPLOY.md
```

**Conformance:**

```bash
lab-kit conformance run   # wraps HelixTest when installed
helixtest --all --mode ferrum+infra --profile ferrum-infra
```

## Documentation map

| Topic | Document |
|-------|----------|
| Ferrum ↔ ga4gh-infra wiring | [Ferrum GA4GH-INFRA-INTEGRATION.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH-INFRA-INTEGRATION.md) |
| Demo compose merge order | [Ferrum-GA4GH-Demo architecture.md](https://github.com/SynapticFour/Ferrum-GA4GH-Demo/blob/main/docs/architecture.md) |
| Co-deploy profiles | [field-edge+infra.toml](../config/profiles/field-edge+infra.toml), [field-edge+solum.toml](../config/profiles/field-edge+solum.toml), [bra-companion.toml](../config/profiles/bra-companion.toml), [institute.toml](../config/profiles/institute.toml) |
| HelixTest co-deploy mode | [helixtest/docs/ferrum.md](https://github.com/SynapticFour/HelixTest/blob/main/helixtest/docs/ferrum.md) |
| Africa-Mode (SQLite) | [ga4gh-infra AFRICA-DEPLOYMENT](https://github.com/SynapticFour/ga4gh-infra/blob/main/docs/AFRICA-DEPLOYMENT.md) |

## CI

GitHub Actions runs `cargo test`, field-edge compose generation (inside the `rust` job), cargo-deny, and ARM64 builds on `main`.
