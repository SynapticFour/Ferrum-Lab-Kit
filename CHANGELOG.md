# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Deployment CLI (`lab-kit`) for profile init, Compose/Helm/systemd generation, status, ingest helpers, and HelixTest conformance wrappers.
- Deployment profiles (`beacon-only`, `field-edge`, `field-edge+infra`, `institute`) and co-deploy fragments for local **ga4gh-infra**.
- Git-pinned `ferrum-core` integration via `lab-kit-ferrum` with bump script and scheduled CI pin workflow.
- Conformance workflow: compose generation for beacon-only / field-edge, HelixTest CLI smoke, and opt-in live suite via `workflow_dispatch`.
- Local pre-commit / `scripts/hooks/ci-check.sh` mirroring primary GitHub CI checks.

### Changed

- Document Compose/Helm service images as **placeholders**; Ferrum GHCR currently publishes monolith gateway/UI images only (see README and `docs/FERRUM-INTEGRATION.md`).
- Conformance live HelixTest step no longer masks failures with `|| true` when the opt-in suite is run.
- `TESTING.md` and `RELEASING.md` describe what CI actually gates and the intended first SemVer tag.

### Fixed

- Ship-hygiene documentation no longer claims aspirational quality gates that do not exist.

### Security

- Secret-scan and dependency-review workflows retained; stub quality-gate workflow removed in favor of real checks.

## [0.1.0-alpha] — unreleased tag

Intended first SemVer tag when release artifacts (verified images/docs) are ready. **Do not cut `v0.1.0-alpha` until those artifacts exist** — see [RELEASING.md](RELEASING.md).
