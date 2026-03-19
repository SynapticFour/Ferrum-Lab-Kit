# Business model (open core)

Licensed under **BUSL-1.1** with parameters aligned to [Ferrum](https://github.com/SynapticFour/Ferrum) — see [LICENSE](../LICENSE). **Change Date:** four years from each version’s release (same rule as Ferrum). Non-commercial research and academic use is permitted under the **Additional Use Grant**.

## Free (open source under BUSL terms)

- Selective GA4GH deployments via Lab Kit (DRS, WES, TES, TRS, Beacon v2, htsget, …).
- ELIXIR LS Login integration (`lab-kit-auth`).
- All three deployment targets (Compose, Helm, systemd/HPC).
- Adapter traits and built-in S3 / POSIX / SLURM / SQLite paths.
- HelixTest **runner hook** and **JSON** conformance report.
- Documentation and example profiles.

## Commercial (Synaptic Four)

- **PDF conformance report** generation (`lab-kit-report`) — requires **`FERRUM_LAB_KIT_LICENSE_KEY`** at runtime. Rationale: high-value artifact for grants and consortium submissions.
- **Multi-site federation tooling** (Beacon Network across deployments) — planned product boundary.
- **Managed deployment & sign-off** — consulting engagement.
- **Priority support SLA** — paid support channel.

## Non-negotiable

The **license key must never gate** running a conformant GA4GH stack. Only the **PDF report output** (and future paid-only tooling) is gated.
