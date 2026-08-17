# Business model (open core)

Licensed under **BUSL-1.1** with parameters aligned to [Ferrum](https://github.com/SynapticFour/Ferrum) — see [LICENSE](../LICENSE). **BUSL is not an OSI Open Source license.** **Change Date:** four years from each version’s first public distribution. Non-commercial research and academic use is permitted under the **Additional Use Grant**.

## Free (source-available under BUSL terms)

- Selective GA4GH **deployment generation** (Compose, Helm, systemd) around the Ferrum monolith.
- Config schema, `lab-kit-auth` helpers, adapter **libraries** (not Ferrum runtime).
- HelixTest **runner hook** and **JSON** conformance report (empty / skip-only results are **not** a pass).
- Documentation and example profiles.

## Commercial (Synaptic Four)

- **PDF conformance report** (`lab-kit-report`) — requires a **signed** Ed25519 token `flk1.<payload>.<sig>` in **`FERRUM_LAB_KIT_LICENSE_KEY`** and **`lab-kit license activate`**. Unsigned `flk_` prefixes and SHA-256 self-activation are rejected. Verify with `FERRUM_LAB_KIT_LICENSE_PUBKEY` (64-char hex) or `keys/license-ed25519.pub`.
- **Multi-site federation tooling** (Beacon Network across deployments) — planned product boundary.
- **Managed deployment & sign-off** — consulting engagement.
- **Priority support SLA** — paid support channel.

Lab Kit **comes with Ferrum**. It is not a fifth SKU. Written Ferrum commercial license: [Ferrum COMMERCIAL.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/COMMERCIAL.md).

## Non-negotiable

The **license key must never gate** running a GA4GH stack. Only the **PDF report output** (and future paid-only tooling) is gated.
