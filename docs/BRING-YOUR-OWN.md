# Bring your own infrastructure

Lab Kit **generates** Compose/Helm/systemd around the Ferrum monolith. POSIX/S3/SLURM settings in `lab-kit.toml` are **written onto Ferrum env** (`FERRUM_STORAGE__*`, `FERRUM_WES_BACKEND`, `FERRUM_TES_BACKEND`). The adapter **crates** are not linked into the Ferrum binary. Use `external_url` where Ferrum should call an existing endpoint instead of deploying that surface.

## External service URLs

In `lab-kit.toml`, set `external_url` on any `[services.*]` block to **skip** deploying that Ferrum surface and point integrations at your existing endpoint:

```toml
[services.drs]
external_url = "https://drs.your-institute.de"
```

The **service registry** marks `deploy: false` and uses your URL for health pre-checks where applicable.

### `external-upstreams.yaml` (Compose)

When you run `lab-kit generate compose`, Lab Kit writes **`external-upstreams.yaml`** next to the merged `docker-compose.yml` if any service uses `external_url`. It lists **service → base URL** for your reverse proxy (Traefik, Caddy, Envoy, …) so traffic to local routes can be forwarded to existing institute endpoints. See the `note` field inside the generated file.

### `proxy-traefik-dynamic.yaml` (Traefik)

In addition, Lab Kit generates **`proxy-traefik-dynamic.yaml`** (when any service is external) with a ready-to-load Traefik *dynamic config*.

Path prefixes match **[Ferrum `ferrum-gateway`](https://github.com/SynapticFour/Ferrum)** (e.g. `/ga4gh/drs/v1`, `/ga4gh/trs/v2`, `/ga4gh/beacon/v2`, `/ga4gh/htsget/v1`, `/passports/v1`, …) and forward to each service’s `external_url` base.

**No Traefik yet?** See the optional generic Compose stack under [`deploy/traefik/`](../deploy/traefik/README.md) (copy the generated YAML into `deploy/traefik/dynamic/`).

## Adapter traits (`lab-kit-adapters`)

These traits live in **this repository**. They are **not** linked into the Ferrum binary. `lab-kit generate` maps POSIX/S3/SLURM onto Ferrum env so the gateway uses those backends. Use `lab-kit adapters check` to probe what Lab Kit can see on the operator machine (POSIX write, SQLite ping, `sbatch` on PATH). It does **not** submit SLURM jobs or run Nextflow.

| Trait | Purpose in Lab Kit |
|-------|-------------------|
| `StorageBackend` | S3/MinIO (`S3StorageBackend`), POSIX (`PosixStorageBackend`) — library + CLI probe |
| `ComputeBackend` | SLURM presence check (`sbatch --version`); SSH config is recorded, not connected by default |
| `MetadataStore` | SQLite (`SqliteMetadataStore`) and PostgreSQL (`PostgresMetadataStore`) via **sqlx** + embedded migrations |
| `WorkflowEngine` | `NextflowWorkflowEngine` is a **stub**: it prints that pipelines belong to Ferrum WES |

S3 keys in `lab-kit.toml` are **not** copied into Compose. Set `FERRUM_S3_ACCESS_KEY_ID` / `FERRUM_S3_SECRET_ACCESS_KEY` in the environment.

## Bring your own hardware (Raspberry Pi / ARM64)

**Ferrum Lab Kit** and **Ferrum** support **ARM64** (Raspberry Pi 5, Apple Silicon, ARM cloud) and **x86_64**. Use the **`field-edge`** profile:

```bash
./install-edge.sh
# or interactively:
lab-kit init   # → select "Field / Edge"
```

This profile uses **SQLite + local filesystem**, disables heavyweight services by default, and applies `deploy/docker-compose/edge.yml` with memory/power env that **stock Ferrum ignores** unless the pinned image implements them.

See [Deployment targets — Field / Edge](DEPLOYMENT-TARGETS.md#field-edge).

## Global external shortcuts

```toml
[external]
htsget_url = "https://htsget.your-institute"
beacon_network_url = "https://beacon-network.elixir-europe.org"
```

Use these for documentation and downstream templates (Beacon Network registration, etc.).
