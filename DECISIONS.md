# Engineering Decisions (ADR-lite)

Track important architectural and operational decisions here.

## Template

### YYYY-MM-DD - Decision title

- **Context:** Why this decision was needed.
- **Decision:** What was chosen.
- **Consequences:** Trade-offs, risks, and follow-up actions.

---

### 2026-08-15 - Ferrum image variants, not a Lab Kit compiler

- **Context:** Operators want Beacon+DRS (or field+infra) without pulling the full WES/TES/TRS binary, and sometimes a local image for arm64.
- **Decision:** Lab Kit **selects** Ferrum GHCR tags `:<sha>`, `:<sha>-edge`, `:<sha>-edge-infra` from the profile. `lab-kit build image` wraps Ferrum’s Dockerfile for a custom platform. Lab Kit does not invent per-surface images.
- **Consequences:** Needs Ferrum GHCR to publish those tags. htsget stays in the `edge` binary (disabled at runtime for `field-edge`). `--legacy-per-service` remains unpublished placeholders.

---

### 2026-08-18 - Adapter config is wired into Ferrum env at generate time

- **Context:** `lab-kit-adapters` traits were libraries plus `adapters check` only. Compose already mapped POSIX/S3/SLURM onto `FERRUM_*` env; Helm did not. Operators could believe adapters were unused.
- **Decision:** `lab-kit generate` (Compose **and** Helm) writes adapter settings onto the Ferrum gateway env (`FERRUM_STORAGE__*`, `FERRUM_WES_BACKEND`, `FERRUM_TES_BACKEND`, SLURM partitions). Ferrum still owns runtime I/O. The traits stay in Lab Kit for `adapters check` and as the config schema. Ferrum does not take a crate dependency on Lab Kit.
- **Consequences:** A Lab Kit profile with POSIX DRS or SLURM WES/TES produces a gateway that Ferrum actually honours. Helm values include `gateway.adapters`. S3 keys stay in the environment, not in values YAML.

### 2026-08-15 - Runtime is the Ferrum monolith, not Lab Kit adapters

- **Context:** Marketing and README described Lab Kit as a BYO-SLURM/S3/OIDC platform. The CLI runtime path only generates Compose/Helm/systemd around `ghcr.io/synapticfour/ferrum`.
- **Decision:** Treat Lab Kit as a **YAML on-ramp**. `lab-kit-adapters` / `lab-kit-auth` stay in-tree as libraries and `lab-kit adapters check`. Ferrum images own GA4GH I/O. Docs must state this boundary.
- **Consequences:** Honest onboarding; no false expectation that Ferrum links Lab Kit traits. Per-service compose/helm images remain `--legacy-per-service` only.
- **Superseded in part (2026-08-18):** generate-time env wiring is now in both Compose and Helm. Ferrum still does not crate-depend on Lab Kit.

### 2026-08-15 - Auth and Passport enforcement belong to Ferrum

- **Context:** Lab Kit has OIDC/Passport helpers but does not run the browser flow (no PKCE, no code exchange). LDAP was listed as a provider but never implemented.
- **Decision:** Config rejects `auth.provider = "ldap"`. Local/offline uses Ferrum’s Passport path. Visa JWKS fetch is fail-closed (issuer allowlist + TTL). Controlled without grant is **Denied**.
- **Consequences:** Operators register OIDC clients against the Ferrum gateway callback. Lab Kit only writes config and env.

### 2026-08-15 - Open-core PDF licenses are signed tokens

- **Context:** PDF gating previously accepted any `flk_` prefix plus SHA-256 self-activation.
- **Decision:** Tokens are `flk1.<payload>.<sig>` (Ed25519). Verify with operator-supplied public key. Empty/skip-only HelixTest JSON is not a pass. Unparseable expiry is fail-closed.
- **Consequences:** No issuing private key in the repository. Operators mint keys out of band (`lab-kit-report` example `gen_license_keypair`).

### 2026-04-10 - Establish cross-repo quality and security baseline

- **Context:** Repositories had uneven governance and CI security posture.
- **Decision:** Standardize governance docs, quality gates, and security scanning workflows.
- **Consequences:** Better consistency and contributor trust; ongoing maintenance required to keep checks aligned with stack changes.
