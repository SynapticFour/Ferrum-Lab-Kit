# Ferrum Lab Kit

Generates Compose / Helm / systemd around a SHA-pinned [Ferrum](https://github.com/SynapticFour/Ferrum) monolith image (`ghcr.io/synapticfour/ferrum`). This repository does **not** implement GA4GH protocol logic. Runtime Beacon/DRS/WES I/O is the Ferrum container. Pin: `config/ci/ferrum-revision.txt` (Ferrum **v0.3.2**).

**Maturity: Early access.** Ferrum companion — not a fifth product.

> This README describes technical capabilities, not legal advice.

These public repositories are maintained by the same organisation and are designed to work together. Each repository keeps its own version and license. For details on roles, maturity, and how the components relate to one another, see [SUITE-OVERVIEW](https://github.com/SynapticFour/.github/blob/main/profile/SUITE-OVERVIEW.md).

## Quick start

```bash
make up
```

Stop: `make down`. Remove volumes: `make destroy`. Optional: `make up-with-infra`, `make up-with-solum`, `make up-with-infra-solum`.

## Documentation

- [Getting started](docs/GETTING-STARTED.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Ferrum integration](docs/FERRUM-INTEGRATION.md) · [Documentation index](docs/README.md)

Adapter traits (`lab-kit-adapters`) and OIDC helpers (`lab-kit-auth`) are Lab Kit libraries. `lab-kit generate` maps POSIX/S3/SLURM onto Ferrum gateway env (Compose and Helm). Probe with `lab-kit adapters check`. Ferrum does not take a crate dependency on these libraries.

## License

Business Source License 1.1 — see [LICENSE](LICENSE).
