# Ferrum Lab Kit

Ferrum Lab Kit writes the start files for a Ferrum service that is already built, so an institution can bring that service up on its own computers without reimplementing the interfaces.

For an engineer who deploys Ferrum, and for an evaluator of that deployment.

**Status: Early use.** Registry: TOOL-LAB-KIT-STATUS-001. No public certificate comes with that label.

Ferrum Lab Kit writes Compose, Helm, and systemd around a pinned image. It does not hold the interface logic. Helix checks a running service. HELIOS writes signed evidence. This repository does neither.

## What it does

- It generates Compose, Helm, and systemd around a SHA-pinned Ferrum image (`ghcr.io/synapticfour/ferrum`). Runtime Beacon, DRS, and WES behaviour is that image. This repository does not implement the GA4GH protocol logic. Registry: TOOL-LAB-KIT-USE-001.
- The pin in [`config/ci/ferrum-revision.txt`](config/ci/ferrum-revision.txt) is Ferrum tag **v0.3.2** (`2bd147c99735866ac9cfb530f62ef984c8efce8b`). That pin is deliberate. It is the file on this commit, not a promise that a later commit keeps the same Ferrum tag. It is not a statement that Ferrum's default branch is v0.3.2. Ferrum's default branch is newer. The pin is not "current Ferrum is v0.3.2".

## Limits

- This repository does not implement DRS, WES, TES, TRS, Beacon, htsget, or Crypt4GH.
- Early use is the public label for the older "early access" line. It is not an official seal.
- The workflow file on the main branch tests the program and generates start files for three presets (`field-edge`, `bra-companion`, `federated-node`). This page does not record a run of that workflow.
- The Raspberry Pi path is [docs/RASPBERRY-PI.md](docs/RASPBERRY-PI.md). This page does not record a run on a Pi.

> This README describes technical capabilities, not legal advice.

## Quick start

Prerequisites: Rust, as pinned in `rust-toolchain.toml`, for the check that stays inside this repository. Docker Engine and Docker Compose for the stack. The stack uses the pinned Ferrum image. It does not compile Ferrum.

```bash
cargo test --workspace
```

You should see cargo print `test result: ok`.

The stack needs Docker:

```bash
make up
```

You should see the gateway answer at `http://127.0.0.1:8080/health`. Stop with `make down`. Remove volumes with `make destroy`.

Optional companions in this repository, on the same pin: `make up-with-infra`, `make up-with-solum`, and `make up-with-infra-solum`. Those companions are [ga4gh-infra](https://github.com/SynapticFour/ga4gh-infra) and [Solum](https://github.com/SynapticFour/Solum). This tool does not start them from their own repositories.

## Documentation

The full index, one line for every file under `docs/`, is [docs/README.md](docs/README.md).

### Tutorial

- [Getting started](docs/GETTING-STARTED.md)

### How-to guides

- [Raspberry Pi](docs/RASPBERRY-PI.md)
- [Operations checklist](docs/OPERATIONS-CHECKLIST.md)
- [Bring your own endpoints](docs/BRING-YOUR-OWN.md)
- [Solum beside Lab Kit](docs/SOLUM-CO-DEPLOY.md)
- Further task pages are in the [index](docs/README.md)

### Reference

- [Image pin](docs/IMAGE-PIN-POLICY.md)
- [Ferrum integration](docs/FERRUM-INTEGRATION.md)
- [Changelog](CHANGELOG.md)

### Explanation

- [Architecture](docs/ARCHITECTURE.md)
- [For evaluators](docs/FOR-EVALUATORS.md)
- [Where it aims to run](docs/DEPLOYMENT-TARGETS.md)

## Licence

Ferrum Lab Kit is free under the Apache License, Version 2.0. The file is [LICENSE](LICENSE). Versions released before this change were published under the Business Source License 1.1; see [LICENSING-HISTORY.md](LICENSING-HISTORY.md).

The pinned Ferrum version was released under BUSL-1.1; see Ferrum's [LICENSING-HISTORY.md](https://github.com/SynapticFour/Ferrum/blob/main/LICENSING-HISTORY.md).

Paid are services and commitments. Prices are on the [consulting page](https://synapticfour.com/en/consulting/).

## Cite

Use [CITATION.cff](CITATION.cff). Prefer that file over a hand-copied line.

## Website

- Product page: [https://synapticfour.com/en/ferrum/](https://synapticfour.com/en/ferrum/)
- Technical profile: [https://synapticfour.com/en/technical/ferrum/](https://synapticfour.com/en/technical/ferrum/)

The same pages exist in German (`/de/ferrum/`, `/de/technik/ferrum/`) and in French (`/fr/ferrum/`, `/fr/technique/ferrum/`).

The map of repositories: [profile/README.md](https://github.com/SynapticFour/.github/blob/main/profile/README.md).

Contact: [contact@synapticfour.com](mailto:contact@synapticfour.com)

A German reading of the older long page is [README.de.md](README.de.md). The English page is the base. The German page keeps the command tables.
