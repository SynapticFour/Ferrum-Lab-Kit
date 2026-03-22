# Ferrum integration

Lab Kit depends on the **[Ferrum](https://github.com/SynapticFour/Ferrum)** platform as **library code**, not as a fork.

## Git pin (`ferrum-core`)

- Crate: **`lab-kit-ferrum`** → `ferrum-core` from  
  `https://github.com/SynapticFour/Ferrum.git` pinned by **full git `rev`** (see `crates/lab-kit-ferrum/Cargo.toml`).
- **Bump procedure:** pick a Ferrum commit (often `main` HEAD), set the same SHA in `Cargo.toml` and **`config/ci/ferrum-revision.txt`**, then `cargo update -p ferrum-core` and run tests.
- **Script:** `./scripts/bump-ferrum.sh` updates `Cargo.toml`, `FERRUM_GIT_REV`, and `ferrum-revision.txt` from **`refs/heads/main`** (or pass an explicit 40-char SHA). Use `./scripts/bump-ferrum.sh --dry-run` to preview. Then run `cargo update -p ferrum-core` and `cargo test --workspace`.

## CLI check

```bash
cargo run -p lab-kit-selector -- ferrum check
```

Prints the linked `ferrum_core::FerrumError` type name and the pinned revision.

## Runtime wiring

Deploy generators still use **placeholder images** until you point Compose/Helm at Ferrum release images or build from the Ferrum repo. Shared **types, config, and auth** primitives come from `ferrum-core` via `lab-kit-ferrum` for gateways and future glue code.

**GA4GH local demo (WES + TES Docker, workdirs, `docker.sock`, optional Crypt4GH):** upstream merge overlay and env checklist — see [FERRUM-GA4GH-DEMO-OVERLAY.md](FERRUM-GA4GH-DEMO-OVERLAY.md) and `contrib/ferrum/`.

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
