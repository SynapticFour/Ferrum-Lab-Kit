# Operations checklist (Lab Kit & GA4GH stacks)

Use this as a **runbook scaffold** when bringing up or handing over an environment. **Adapt** rows to your actual Ferrum / Lab Kit deployment — do not treat empty boxes as “required everywhere”.

## Who needs which variables / tools?

| Path | What you use | Env / tooling (this repo + ecosystem) |
|------|----------------|----------------------------------------|
| **Open-source Ferrum + Lab Kit** | Deploy (`generate compose` / Helm), **`lab-kit status`**, **`lab-kit ingest`**, **`lab-kit ferrum check`** | **`FERRUM_GATEWAY_URL`**, **`FERRUM_TOKEN`** only when the gateway requires auth for ingest; Ferrum server config from **your** Ferrum install (see [Ferrum `INSTALLATION.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/INSTALLATION.md)). **No** Lab Kit license key required for GA4GH workflows or JSON reports. |
| **HelixTest conformance** | **`lab-kit conformance run`** | External **[HelixTest](https://github.com/SynapticFour/HelixTest)** CLI; optional **`HELIXTEST_BIN`**. Ferrum CI matrix: [Ferrum `HELIXTEST-INTEGRATION.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/HELIXTEST-INTEGRATION.md). |
| **Licensed PDF reports** | **`lab-kit conformance report`** → PDF | Optional **`FERRUM_LAB_KIT_LICENSE_KEY`** — **only** gates **PDF** output; JSON report is not license-gated ([CONFORMANCE.md](CONFORMANCE.md), [BUSINESS-MODEL.md](BUSINESS-MODEL.md)). |

**Authoritative API paths and auth:** [Ferrum `docs/GA4GH.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md). **TES + Docker:** [`TES-DOCKER-BACKEND.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/TES-DOCKER-BACKEND.md).

---

## A. Naming and documentation hygiene

| Check | Notes |
|-------|--------|
| ☐ **Consistent prefixes in docs** | Lab Kit documents **`FERRUM_GATEWAY_URL`**, **`FERRUM_TOKEN`**, and **`FERRUM_LAB_KIT_LICENSE_KEY`** (PDF reports only). Ferrum server config often uses **`FERRUM_…`** with **`__` nested keys** in env form (e.g. auth flags referenced in [FERRUM-INTEGRATION.md](FERRUM-INTEGRATION.md)). Avoid mixing similar names without defining them once in your internal runbook. |
| ☐ **YAML / Compose service names** | Same hostname in Traefik rules, health checks, and client `base URL` examples. |
| ☐ **Version pins** | Note **Ferrum git SHA** (`config/ci/ferrum-revision.txt`, `lab-kit ferrum check`) and **image tags** you actually run. |

---

## B. Container runtime

| Check | Notes |
|-------|--------|
| ☐ **Docker Client vs Daemon API** | Client and server should be **mutually supported** versions; odd failures often trace to API mismatch. |
| ☐ **Nested execution** | If the workflow engine uses **Docker-from-inside-Docker**, confirm **socket** mount and **host paths** for inner `docker run` — pattern in [GA4GH-WORKFLOW-PRIMER.md](GA4GH-WORKFLOW-PRIMER.md) §3; **details:** [Ferrum `TES-DOCKER-BACKEND.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/TES-DOCKER-BACKEND.md). |
| ☐ **`host.docker.internal`** | On some hosts, containers reach the host via this DNS name; on Linux it may be **missing** unless explicitly added. Do not assume macOS semantics on Linux workers. |
| ☐ **Platform (`amd64` / `arm64`)** | Align image pull policy with [GA4GH-WORKFLOW-PRIMER.md](GA4GH-WORKFLOW-PRIMER.md) §5. |

---

## C. Network and URLs

| Check | Notes |
|-------|--------|
| ☐ **Gateway base URL** | Single canonical origin for API clients (no trailing path). Used by **`lab-kit ingest`** (`--gateway` / `FERRUM_GATEWAY_URL` / `[ferrum].gateway_url`). |
| ☐ **TLS / trust** | Internal CA or self-signed certs: clients (CI, notebooks) need matching trust store. |
| ☐ **Egress** | TRS/WES may need outbound HTTPS for descriptor URLs or registries. |

---

## D. Storage and working directories

| Check | Notes |
|-------|--------|
| ☐ **TES / executor workdirs** | Enough disk for staging inputs, engine temp files, and outputs. |
| ☐ **Shared visibility** | Paths visible to **TES tasks** must match what **WES / engine** configuration passes (see primer §3). |
| ☐ **DRS backend** | Object storage endpoint, bucket, and credentials consistent with ingest and access URLs. **DRS routes / access vs stream:** [Ferrum `GA4GH.md` — DRS](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md#drs-data-repository-service). |

---

## E. Environment variables (reference)

**Defined / used by Lab Kit tooling (this repo):**

| Variable | Purpose |
|----------|---------|
| `FERRUM_GATEWAY_URL` | Base URL of ferrum-gateway for **`lab-kit ingest`**. |
| `FERRUM_TOKEN` | Optional Bearer token for authenticated ingest. |
| `FERRUM_LAB_KIT_LICENSE_KEY` | Optional; enables **PDF** conformance reports (`lab-kit-report`). |
| `HELIXTEST_BIN` | Override path to **HelixTest** binary for `lab-kit conformance run`. |

**Ferrum server / deployment:** Many settings come from **`config.toml`** and env overlays documented in **[Ferrum `INSTALLATION.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/INSTALLATION.md)** and **[`GA4GH.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md)**. **Do not guess** `FERRUM_*` names in runbooks — copy from the version you deploy.

---

## F. Lab Kit CLI quick checks

Commands match `crates/lab-kit-selector` (binary name **`lab-kit`**). From repo root you can use **`cargo run -p lab-kit-selector -- <subcommand> …`** instead of `lab-kit …`.

| Step | Command |
|------|---------|
| Config load + health poll | `lab-kit status --config lab-kit.toml` |
| Ferrum `ferrum-core` git pin | `lab-kit ferrum check` |
| Ingest smoke (gateway running) | `lab-kit ingest --gateway <GATEWAY_URL> register-url https://example.com/f.txt --name demo` |
| Conformance driver | `lab-kit conformance run --config lab-kit.toml` ([CONFORMANCE.md](CONFORMANCE.md)) |
| Report from HelixTest JSON | `lab-kit conformance report --helixtest-json <path> --out-dir reports/conformance --config lab-kit.toml` |

**Not provided by Lab Kit:** **`lab-kit wes`** / **`lab-kit tes`** — use HTTP against the gateway; see [Ferrum `GA4GH.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md).

---

## G. Assumptions

- Generated **Compose / Helm** files are **templates**; operators must fill images, secrets, and volumes.  
- **Conformance** tooling (HelixTest) is **external** to this repo — see [CONFORMANCE.md](CONFORMANCE.md).

---

*[← Documentation index](README.md)*
