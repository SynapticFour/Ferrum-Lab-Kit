# Bring your own infrastructure

Lab Kit is designed for labs that **already** run storage, schedulers, and IdPs.

## External service URLs

In `lab-kit.toml`, set `external_url` on any `[services.*]` block to **skip** deploying that Ferrum service and point integrations at your existing endpoint:

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

| Trait | Purpose |
|-------|---------|
| `StorageBackend` | S3/MinIO (`S3StorageBackend`), POSIX (`PosixStorageBackend`), … |
| `ComputeBackend` | SLURM: local login node (`SlurmComputeBackend`, `sbatch`/`squeue`) or remote (`SlurmSshComputeBackend`, `ssh user@login … sbatch`/`squeue`) |
| `MetadataStore` | SQLite (`SqliteMetadataStore`) and PostgreSQL (`PostgresMetadataStore`) via **sqlx** + embedded migrations under `crates/lab-kit-adapters/migrations/{sqlite,postgres}/` |
| `WorkflowEngine` | Nextflow hook (`NextflowWorkflowEngine` defers to Ferrum WES) |

Ferrum services should depend on these traits rather than hard-coding vendors.

## Global external shortcuts

```toml
[external]
htsget_url = "https://htsget.your-institute"
beacon_network_url = "https://beacon-network.elixir-europe.org"
```

Use these for documentation and downstream templates (Beacon Network registration, etc.).
