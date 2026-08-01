# Releasing

This repository follows Semantic Versioning (`MAJOR.MINOR.PATCH`).

## First tag

The intended first SemVer tag is **`v0.1.0-alpha`**.

Do **not** cut that tag (or any release tag) until release artifacts truly exist — at minimum: changelog section closed for the release, CI green on `main`, and deploy image references that operators can actually pull (or an explicit, documented build-from-source path). Compose/Helm still ship **placeholder** per-service image names; Ferrum’s published GHCR images today are the monolith gateway/UI only (see [docs/FERRUM-INTEGRATION.md](docs/FERRUM-INTEGRATION.md)).

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
