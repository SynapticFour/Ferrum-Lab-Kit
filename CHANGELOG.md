# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- **h2 0.4.16+** — cargo-deny RUSTSEC-2026-0258 (unbounded empty DATA frames). Transitive via hyper; lockfile bump only.
- HelixTest CI pin **v0.1.3** (`1832c04`). Ferrum image pin **v0.3.2** (`2bd147c9`).
- Written Ferrum commercial license (Lab Kit is not sold separately): [Ferrum COMMERCIAL.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/COMMERCIAL.md).
- **`federated-node` profile** — Beacon v2 boolean+count, DRS, Passport-on-DRS (ga4gh-infra), service-registry auto-register, Traefik TLS on **8443**. TES/WES off unless you opt in. Compose-up is **not** an EGA/GDI/ELIXIR membership card. [docs/FEDERATED-NODE.md](docs/FEDERATED-NODE.md).
- **`archive-submitter` profile** — Ferrum-edge + Metadata Store, no WES/TES. Writes reviewable GHGA/EGA/H3Africa bundles + DRS IDs via Ferrum `ferrum meta export`. Does **not** upload to EGA. Not a fifth SKU. `lab-kit init --profile archive-submitter --non-interactive`.
- **BRA workbench companion** — `[bra]` config, `bra.yml`, CLI `--with-bra`, profile `bra-companion`. Lab Kit does not ship a BRA image (`BRA_IMAGE` required). Not a combo SKU. [docs/BRA-CO-DEPLOY.md](docs/BRA-CO-DEPLOY.md).
- **SPDX on first-party `.rs`** — `// SPDX-License-Identifier: BUSL-1.1`; CI `spdx.yml`.
- Pin Ferrum **v0.3.2** (`2bd147c9`) for `ferrum-core` and GHCR variant tags (full / edge / edge-infra).
- Solum sidecar pin **v0.1.0** (same tag as Solum-Demo). Historical companion SHA `6b4519c` is retired.
- ga4gh-infra Compose image tags **0.2.3** (`ga4gh-infra-v0.2.3` stack publish).

## [0.2.0-alpha] - 2026-08-15

Named Ferrum image variants so a Beacon/DRS stack can pull a smaller gateway binary. Ferrum pin: `a4ba89911e207b9597e03c321f0e18ea9112d57a`.

### Added

- **Ferrum image variants** — generated Compose/Helm pin `:<sha>-edge` (Beacon/DRS) or `:<sha>-edge-infra` (ga4gh-infra co-deploy) instead of always the full monolith. `lab-kit build image --variant edge --platform linux/arm64` builds from the pinned Ferrum SHA.

## [0.1.0-alpha] - 2026-08-15

First tagged on-ramp: generate Compose/Helm/systemd for a **selected** Ferrum GA4GH subset (one monolith image, `FERRUM_SERVICES__ENABLE_*`). Ferrum pin: `6788bfe11860b5fe49bae72d120373f78a0b023f` (`ghcr.io/synapticfour/ferrum:<sha>`, published by Ferrum’s GHCR workflow on that commit).

### Added

- **`lab-kit adapters check`** — probes POSIX/SQLite locally and reports SLURM/S3/Nextflow configuration without submitting jobs. Ferrum still owns runtime GA4GH I/O.
- **`lab-kit generate infra-secrets`** — local RSA PEMs + `secrets.env` for ga4gh-infra (gitignored). Wired into `install-edge.sh --with-infra`, `scripts/stack-up.sh`, and the Raspberry Pi kit.
- **SHA image pins** — `config/ci/ferrum-image.txt` / `ferrum-image-arm64.txt`; generated Compose/Helm/Pi kit/install-edge default to `ghcr.io/synapticfour/ferrum:<git-sha>`. `./scripts/bump-ferrum.sh` updates the pin files.
- **Signed PDF license tokens** — `flk1.<payload>.<sig>` (Ed25519). Unsigned `flk_` blobs are rejected. No issuing private key in the repository.
- **CI** — `cargo deny check` (`deny.toml`), CodeQL SARIF upload, blocking dependency-review on PRs. Dependabot stays off (same as before the due-diligence pass).
- Engineering ADRs in [DECISIONS.md](DECISIONS.md) (monolith on-ramp, auth ownership, signed licenses).
- **Monolith runtime path** — `docker-compose.gateway.yml` pulls `ghcr.io/synapticfour/ferrum` and injects `FERRUM_SERVICES__ENABLE_*` from the selected profile (usable `docker compose up`).
- **Solum companion** — `[solum]` config, `solum.yml` + `Dockerfile.solum-sidecar`, CLI `--with-solum`, profiles `field-edge+solum` / `field-edge+infra+solum`, `make up-with-solum` / `up-with-infra-solum`, [docs/SOLUM-CO-DEPLOY.md](docs/SOLUM-CO-DEPLOY.md).
- **Raspberry Pi field kit** — `lab-kit generate raspberry-pi` (alias `pi`) writes a portable `pi-kit/` (compose, `.env` ARM64, `install-on-pi.sh`); `make pi-kit`; [docs/RASPBERRY-PI.md](docs/RASPBERRY-PI.md).
- **ga4gh-infra external mode** — `co-deploy-external.yml` (no local broker containers); broker port applied to co-deploy compose.
- **`.env.example`** — Ferrum / ga4gh-infra / Solum variables for compose and CLI.
- Helm `deployment-gateway.yaml` / `deployment-solum.yaml`; systemd emits `ferrum-gateway.service` (+ optional Solum).

### Changed

- README (EN/DE) and operator docs describe Lab Kit as a **Compose/Helm/systemd on-ramp**, not a BYO-SLURM/S3/OIDC runtime inside Ferrum.
- Official MariaDB **BUSL-1.1** license text. BUSL is not described as OSI open source.
- `lab-kit conformance run` always invokes HelixTest with `--all --mode ferrum` and kills the process on timeout.
- systemd units write `gateway.env`; Ferrum does not read `lab-kit.toml`.
- Helm `values.yaml` gateway image uses the SHA pin. Legacy per-service `:latest` images remain unpublished placeholders (`--legacy-per-service`). Solum `lab-kit` tag is a local compose-build name, not GHCR.
- Default compose/Helm/systemd no longer depend on unpublished per-service images; `--legacy-per-service` retains the old fragments.
- Health checks and edge installer wait on gateway port **8080**.
- Ferrum `ferrum-core` and container image pins moved to `6788bfe11860b5fe49bae72d120373f78a0b023f`. `./scripts/bump-ferrum.sh` also rewrites operator defaults (`.env.example`, gateway compose, Helm values, `install-edge` fallback).

### Security

- ga4gh-infra compose requires env secrets (`${VAR:?…}`); no committed `dev-*` defaults.
- Generated Compose emits `${FERRUM_S3_ACCESS_KEY_ID}` / `${FERRUM_S3_SECRET_ACCESS_KEY}` instead of copying keys from TOML.
- Visa JWKS fetch is fail-closed (issuer allowlist + 10-minute TTL). Controlled access without a grant is **Denied**.
- `auth.provider = "ldap"` is rejected at config validate.
- Ingest HTTP client uses a 30-second timeout.

### Fixed

- Empty or skip-only HelixTest JSON is not a conformance pass. Unparseable license expiry is fail-closed.
- Field-edge profile environment is `field`; `auth.mode` is accepted as an alias for `auth.provider`.
- Edge / co-deploy overlays now apply to a real `ferrum-gateway` service (previously orphaned patches).

[0.2.0-alpha]: https://github.com/SynapticFour/Ferrum-Lab-Kit/releases/tag/v0.2.0-alpha
[0.1.0-alpha]: https://github.com/SynapticFour/Ferrum-Lab-Kit/releases/tag/v0.1.0-alpha
