# Ferrum integration

Lab Kit depends on the **[Ferrum](https://github.com/SynapticFour/Ferrum)** platform as **library code**, not as a fork.

## Git pin (`ferrum-core`)

- Crate: **`lab-kit-ferrum`** → `ferrum-core` from
  `https://github.com/SynapticFour/Ferrum.git` pinned by **full git `rev`** (see `crates/lab-kit-ferrum/Cargo.toml`).
- **Bump procedure:** pick a Ferrum commit (often `main` HEAD), set the same SHA in `Cargo.toml` and **`config/ci/ferrum-revision.txt`**, then `cargo update -p ferrum-core` and run tests.
- **Script:** `./scripts/bump-ferrum.sh` updates `Cargo.toml`, `config/ci/ferrum-revision.txt`, and the **image pin files** (`config/ci/ferrum-image.txt`, `ferrum-image-arm64.txt`) from **`refs/heads/main`** (or pass an explicit 40-char SHA). Use `./scripts/bump-ferrum.sh --dry-run` to preview. Then run `cargo update -p ferrum-core` and `cargo test --workspace`. `lab-kit ferrum check` reads the revision from that text file.

## CLI check

```bash
cargo run -p lab-kit-selector -- ferrum check
```

Prints the linked `ferrum_core::FerrumError` type name and the pinned revision.

## Runtime wiring / container images

**Default path (usable today):** `lab-kit generate compose` merges the **monolith** gateway fragment and sets selective surfaces with Ferrum env flags:

| Lab Kit selection | Injected env |
|-------------------|--------------|
| `[services.beacon]` | `FERRUM_SERVICES__ENABLE_BEACON=true` |
| `[services.drs]` | `FERRUM_SERVICES__ENABLE_DRS=true` |
| … | `ENABLE_HTSGET` / `WES` / `TES` / `TRS` |

| Package | Typical tags |
|---------|----------------|
| `ghcr.io/synapticfour/ferrum` | SHA tag in `config/ci/ferrum-image.txt` (do not float `:latest` for generated artefacts) |
| `ghcr.io/synapticfour/ferrum-ui` | same scheme (optional UI; not required by Lab Kit) |

Override with `FERRUM_IMAGE` / `FERRUM_PORT` (see [`.env.example`](../.env.example)).

**Legacy path:** `--legacy-per-service` still emits unpublished `synapticfour/ferrum-beacon` (etc.) fragments for operators who build their own multi-container images. Those tags are **not** on GHCR today — see `deploy/docker-compose/legacy/README.md`.

**GA4GH local demo (WES + TES Docker, workdirs, `docker.sock`, optional Crypt4GH):** upstream merge overlay and env checklist — see [FERRUM-GA4GH-DEMO-OVERLAY.md](FERRUM-GA4GH-DEMO-OVERLAY.md) and `contrib/ferrum/`.

### ga4gh-infra co-deploy

```bash
lab-kit generate compose --with-ga4gh-infra …
# or [ga4gh_infra] mode = "co-deploy" | "external"
make up-with-infra
```

Merges vendored `infra.yml` (GHCR ga4gh-infra images) + `co-deploy.yml` (JWKS / discovery on `ferrum-gateway`). External mode uses `co-deploy-external.yml` and your `service_registry_url` / broker port — no local broker containers.

### Solum companion

Optional consent sidecar — see [SOLUM-CO-DEPLOY.md](SOLUM-CO-DEPLOY.md).

## Versioned ingest (`/api/v1/ingest/*`)

Ferrum exposes a **stable, scripting-oriented** ingest API on **ferrum-gateway** (same auth as other gateway routes). Upstream specification:

- **[Ferrum `docs/INGEST-LAB-KIT.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/INGEST-LAB-KIT.md)** — paths, JSON shapes, multipart upload, idempotency (`client_request_id`), structured errors.

Lab Kit ships:

| Piece | Role |
|-------|------|
| **`lab-kit-ingest`** | Rust client: `register`, `upload` (multipart), `get_job` |
| **`lab-kit ingest`** | CLI wrapping that client |

### Configuration

- **Gateway URL:** `--gateway`, environment **`FERRUM_GATEWAY_URL`**, or optional **`[ferrum].gateway_url`** in `lab-kit.toml` (see `config/lab-kit.example.toml`).
- **Bearer token:** `--token` or **`FERRUM_TOKEN`** when `FERRUM_AUTH__REQUIRE_AUTH=true` (see Ferrum installation docs).

If `lab-kit.toml` is missing or invalid, you can still run `lab-kit ingest` when **`--gateway` or `FERRUM_GATEWAY_URL`** is set.

### CLI examples

```bash
# Register one URL (demo gateway, no token)
lab-kit ingest --gateway http://localhost:8080 register-url https://example.com/data.txt --name demo

# Full register body from JSON (see upstream doc; repo example below)
lab-kit ingest --gateway http://localhost:8080 register --json config/examples/ingest-register.json

# Multipart upload
lab-kit ingest --gateway http://localhost:8080 upload --file ./README.md

# Poll job
lab-kit ingest --gateway http://localhost:8080 job <job_id>
```

Verify objects with DRS: `GET {gateway}/ga4gh/drs/v1/objects/{id}`.

### Library use

Other Rust tools in your workspace can depend on **`lab-kit-ingest`** and call `IngestClient` directly.

## Optional MII Connect wrappers

Lab Kit exposes lightweight passthrough commands to upstream Ferrum MII Connect:

```bash
# Regenerate manifest from pinned packages (delegates to ferrum)
lab-kit mii sync-manifest \
  --spec profiles/mii/sync-spec.json \
  --output profiles/mii/manifest.json \
  --cache-dir profiles/mii/package-cache

# Validate payload against vendored manifest (delegates to ferrum)
lab-kit mii validate \
  --input ./etl-output/fhir \
  --manifest profiles/mii/manifest.json \
  --strict \
  --format text
```

MII remains **optional** — Lab Kit stays GA4GH-centric; clinical MII ownership is Ferrum MII Connect.
