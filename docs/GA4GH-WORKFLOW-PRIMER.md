# GA4GH workflows — concepts for reproducible genomics

**Audience:** People who already know genomics formats and analysis steps, and want **GA4GH-style services** (DRS, TRS, WES, TES, …) to fit into a **reproducible** mental model. **Which env vars / tools for open-source vs HelixTest vs PDF:** [OPERATIONS-CHECKLIST.md](OPERATIONS-CHECKLIST.md) (section *Who needs which variables / tools?*).

**Scope:** This document is **Lab Kit–oriented teaching material**. **Authoritative endpoints, auth rules, and executor behaviour** live in **[Ferrum `docs/GA4GH.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md)** (single source of truth). Below: **patterns only** + links (“details im Core”).

| Topic (details upstream) | Ferrum doc |
|---------------------------|------------|
| DRS metadata vs `/access/{access_id}` vs `/stream` | [GA4GH.md — DRS](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md#drs-data-repository-service) |
| TES + Docker/Podman, sockets, entrypoints | [`TES-DOCKER-BACKEND.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/TES-DOCKER-BACKEND.md) |
| WES/TES/TRS paths, WES limitations | [GA4GH.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md) (per-service sections) |
| Workflow engines (submission, DRS integration) | [`WORKFLOWS.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/WORKFLOWS.md) |
| Versioned ingest (`/api/v1/ingest/*`) | [`INGEST-LAB-KIT.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/INGEST-LAB-KIT.md) |

---

## 1. End-to-end control flow (typical pattern)

Think in **layers**: descriptors → run metadata → tasks → containers.

| Phase | GA4GH role (typical) | What happens (conceptually) |
|-------|----------------------|-----------------------------|
| **Discover** | **TRS** (Tool Registry Service) | You resolve a **workflow or tool descriptor** (e.g. WDL, CWL, Nextflow) to a versioned artefact (file content, Git ref, or packaged tool). |
| **Prepare inputs** | **DRS**, HTTPS, POSIX | Inputs are **located**: object IDs (DRS), direct URLs, or paths visible to the **compute environment** that will run tasks. |
| **Submit a run** | **WES** (Workflow Execution Service) | You create a **workflow run**: descriptor reference + parameterization (inputs, engine hints, resource tags). The service records state and run IDs. |
| **Execute work** | **TES** (Task Execution Service) | Individual **tasks** (command + container image + mounts + env) are **scheduled** on a backend (local Docker, Kubernetes, HPC batch, …). |
| **Inside the task** | *Workflow engine* | Often a **workflow engine** (see §4) runs **inside** a container or host environment and **spawns further** processes or containers **per step**. |

**Important:** WES describes *workflow runs*; TES describes *tasks*. In many stacks, a WES implementation **translates** a workflow into one or many TES tasks (or uses an intermediate layer). The exact mapping is **product-specific** — treat the table above as a **portable pattern**, not a guarantee of how every deployment orders internal calls. **Ferrum:** see [GA4GH.md — WES / TES](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md).

---

## 2. DRS: metadata vs bytes

**Pattern:** clients first fetch **object metadata** (`GET …/objects/{id}`), then obtain **bytes** via **`/access/{access_id}`** and/or **`/stream`** as documented for your deployment — not by treating the JSON document as file contents.

**Single source of truth:** method/path table and Ferrum-specific notes — **[Ferrum `docs/GA4GH.md` — DRS](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md#drs-data-repository-service)**. Crypt4GH / header re-wrap: **[`CRYPT4GH.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/CRYPT4GH.md)**, routing notes **[`drs-crypt4gh-routing.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/drs-crypt4gh-routing.md)**.

**Why engines care (pattern only):** localization must end in a **path or URL the task can read**. Confusing metadata JSON with payload bytes is a common failure mode.

---

## 3. Nested execution: engine + task containers + Docker socket

**Pattern:** an **outer** runtime (host or container) exposes **Docker** to a **workflow engine**; **inner** task containers use **host-visible bind paths** — a path that exists only *inside* the outer container may not exist on the host (classic “only `/data` mounted inside” trap). Mitigations: **symmetric host/container paths** or **explicit host paths** in executor config.

**Single source of truth for Ferrum TES + Docker/Podman:** **[`TES-DOCKER-BACKEND.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/TES-DOCKER-BACKEND.md)**. Cross-check WES/engine wiring in **[`WORKFLOWS.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/WORKFLOWS.md)**.

**Lab Kit / Compose:** generated fragments are **starting points** — validate volumes and socket exposure against your host and policy.

---

## 4. Workflow engines (variants, no single winner)

**Pattern:** **WES** is the GA4GH **API** for run submission; **WDL / CWL / Nextflow / …** are **engines** behind concrete deployments. Engine-specific parameters and DRS resolution are **not** duplicated here.

**Details im Core:** **[Ferrum `WORKFLOWS.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/WORKFLOWS.md)** and the **WES** section of **[`GA4GH.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md#wes-workflow-execution-service)** (incl. limitations).

---

## 5. Platform: `linux/amd64` vs `arm64`

Many bioinformatics images and **Bioconda** packages target **`linux/amd64`**. On **Apple Silicon (arm64)** or **arm64 CI**:

- Container pulls may need an explicit **platform** (e.g. Docker `--platform linux/amd64`) or manifest support in your cluster.
- **Performance** and **emulation** (qemu) trade off against compatibility.
- **CI:** Pin image digests or platforms so pipelines do not flip architecture silently between developer laptops and runners.

Document **your** chosen platform in runbooks so teammates reproduce the same environment.

---

## 6. Reproducible happy path (Lab Kit CLI + Ferrum gateway)

Goals: one stack, one gateway base URL, consistent volumes. **Replace paths and URLs** with yours. Commands below match **`lab-kit` from this repo** ([`install.sh`](../install.sh) or `cargo run -p lab-kit-selector -- …`).

| Step | What | Command or action |
|------|------|-------------------|
| 0 | *(Optional)* Build/install CLI | `./install.sh` or `cargo build -p lab-kit-selector --release` (binary `target/release/lab-kit`) |
| 1 | Generate deploy artefacts | `cargo run -p lab-kit-selector -- generate compose --config lab-kit.toml --fragments deploy/docker-compose --output docker-compose.yml` (same flags as [README.md](../README.md); Helm: `generate helm …`) |
| 2 | Start stack | **Not scripted in Lab Kit** — e.g. `docker compose -f docker-compose.yml up -d` after you point images at real Ferrum artefacts (see [README.md](../README.md)). **Ferrum-only demo:** upstream **`ferrum demo start`** (Ferrum CLI/install — not this repo). |
| 3 | Health / registry | `cargo run -p lab-kit-selector -- status --config lab-kit.toml` |
| 4 | Ferrum library pin (dev sanity) | `cargo run -p lab-kit-selector -- ferrum check` |
| 5 | Ingest (optional) | `cargo run -p lab-kit-selector -- ingest --gateway http://127.0.0.1:8080 register-url https://example.com/sample.txt --name demo` **or** `… ingest --gateway … register --json config/examples/ingest-register.json`. See [FERRUM-INTEGRATION.md](FERRUM-INTEGRATION.md) / [Ferrum `INGEST-LAB-KIT.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/INGEST-LAB-KIT.md). Optional: `-c lab-kit.toml` (default `lab-kit.toml`). If `lab-kit` is on `PATH`, omit the `cargo run -p lab-kit-selector --` prefix. |
| 5b | Conformance (optional) | `cargo run -p lab-kit-selector -- conformance run --config lab-kit.toml` (needs **`helixtest`** on `PATH` or **`HELIXTEST_BIN`**); report: `cargo run -p lab-kit-selector -- conformance report --helixtest-json path/to/out.json --out-dir reports/conformance --config lab-kit.toml` — see [CONFORMANCE.md](CONFORMANCE.md). |
| 6 | **WES workflow run** | **No `lab-kit` subcommand.** Use your HTTP client against **`POST /ga4gh/wes/v1/runs`** on the gateway; contract and auth in **[Ferrum `GA4GH.md` — WES](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md#wes-workflow-execution-service)**. |
| 7 | **TES tasks** | **No `lab-kit` subcommand.** Use **`GET /ga4gh/tes/v1/tasks/{id}`** etc. per **[Ferrum `GA4GH.md` — TES](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md#tes-task-execution-service)**; Docker behaviour — [`TES-DOCKER-BACKEND.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/TES-DOCKER-BACKEND.md). |

**Assumptions:** Gateway reachable; auth matches demo vs production; storage consistent between ingest and execution.

**Limits:** Lab Kit does **not** ship Ferrum container images; Compose may list **placeholders** until you configure real images/tags.

---

## 7. Related reading

**Lab Kit:** [GA4GH standards mapping](GA4GH-STANDARDS.md) · [Architecture](ARCHITECTURE.md) · [Deployment targets](DEPLOYMENT-TARGETS.md) · [Operations checklist](OPERATIONS-CHECKLIST.md) · [Ferrum integration](FERRUM-INTEGRATION.md) · [Documentation index](README.md)

**Ferrum (authoritative):** [docs/GA4GH.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/GA4GH.md) · [WORKFLOWS.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/WORKFLOWS.md) · [TES-DOCKER-BACKEND.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/TES-DOCKER-BACKEND.md) · [INGEST-LAB-KIT.md](https://github.com/SynapticFour/Ferrum/blob/main/docs/INGEST-LAB-KIT.md)

---

*[← Documentation index](README.md)*
