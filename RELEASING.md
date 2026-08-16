# Releasing

This repository follows Semantic Versioning (`MAJOR.MINOR.PATCH`).

## First tag

**`v0.1.0-alpha`** was cut on **2026-08-15**. It pins Ferrum `6788bfe11860b5fe49bae72d120373f78a0b023f`.

**`v0.2.0-alpha`** (2026-08-15) pins Ferrum `a4ba89911e207b9597e03c321f0e18ea9112d57a` and selects GHCR variants `:<sha>` (full), `:<sha>-edge`, `:<sha>-edge-infra`.

**main (unreleased)** pins Ferrum **v0.3.1** `f28f27800f1d92c6a76670c760d9beb444c368d6`. If a pull fails, build Ferrum from that SHA (`./scripts/build-variant-image.sh` or `deploy/Dockerfile`) or set `FERRUM_IMAGE`. `lab-kit build image` wraps the same Dockerfile for a local/custom architecture.

Subsequent tags follow the process below. Do **not** tag until CI is green on `main` and image pins match a published (or documented build-from-source) Ferrum revision.

## Release train (portfolio)

When **Ferrum** is tagged: the same week, bump this Lab Kit image/git pin to that Ferrum tag. When **ga4gh-infra** is tagged (`ga4gh-infra-v*`): Ferrum `VERSIONS.lock` and Ferrum-GA4GH-Demo follow; Lab Kit only if the Ferrum image itself moved. Showcase pins **tags that exist on origin/main**. See [Ferrum PORTFOLIO.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/PORTFOLIO.md).

Compose/Helm still ship **placeholder** per-service image names behind `--legacy-per-service`; the default path is the monolith gateway only (see [docs/FERRUM-INTEGRATION.md](docs/FERRUM-INTEGRATION.md)).

## Release process

1. Ensure CI is green on `main`.
2. Update `CHANGELOG.md` with user-visible changes (move Unreleased → versioned section).
3. Create an annotated tag:
   - `git tag -a vX.Y.Z -m "vX.Y.Z"`
4. Push the tag:
   - `git push origin vX.Y.Z`
5. Verify GitHub release artifacts and notes.

## Versioning rules

- `MAJOR`: breaking API/behavior changes
- `MINOR`: backward-compatible features
- `PATCH`: backward-compatible fixes and maintenance

## Backport policy

Security fixes should be backported to actively maintained release lines where feasible.
