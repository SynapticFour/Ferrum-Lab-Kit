# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Monolith runtime path** — `docker-compose.gateway.yml` pulls `ghcr.io/synapticfour/ferrum` and injects `FERRUM_SERVICES__ENABLE_*` from the selected profile (usable `docker compose up`).
- **Solum companion** — `[solum]` config, `solum.yml` + `Dockerfile.solum-sidecar`, CLI `--with-solum`, profiles `field-edge+solum` / `field-edge+infra+solum`, `make up-with-solum` / `up-with-infra-solum`, [docs/SOLUM-CO-DEPLOY.md](docs/SOLUM-CO-DEPLOY.md).
- **Raspberry Pi field kit** — `lab-kit generate raspberry-pi` (alias `pi`) writes a portable `pi-kit/` (compose, `.env` ARM64, `install-on-pi.sh`); `make pi-kit`; [docs/RASPBERRY-PI.md](docs/RASPBERRY-PI.md).
- **ga4gh-infra external mode** — `co-deploy-external.yml` (no local broker containers); broker port applied to co-deploy compose.- **`.env.example`** — Ferrum / ga4gh-infra / Solum variables for compose and CLI.
- Helm `deployment-gateway.yaml` / `deployment-solum.yaml`; systemd emits `ferrum-gateway.service` (+ optional Solum).

### Changed

- Default compose/Helm/systemd no longer depend on unpublished per-service images; `--legacy-per-service` retains the old fragments.
- Health checks and edge installer wait on gateway port **8080**.
- Docs (EN/DE README, integration, deployment, operations, ecosystem) aligned with the monolith + companion model.

### Fixed

- Edge / co-deploy overlays now apply to a real `ferrum-gateway` service (previously orphaned patches).

## [0.1.0-alpha] — unreleased tag

Intended first SemVer tag when release artifacts (verified images/docs) are ready. **Do not cut `v0.1.0-alpha` until those artifacts exist** — see [RELEASING.md](RELEASING.md).
