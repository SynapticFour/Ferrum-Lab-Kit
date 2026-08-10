# Legacy per-service Compose fragments

These files (`docker-compose.{beacon,drs,…}.yml`) model a future multi-container
layout with unpublished image names (`synapticfour/ferrum-beacon`, …).

**Default Lab Kit path** uses the monolith gateway
(`../docker-compose.gateway.yml` + `FERRUM_SERVICES__ENABLE_*`).

Opt in only when you build/push your own per-service images:

```bash
lab-kit generate compose --legacy-per-service …
```

Do not expect `docker compose up` to pull the placeholder tags from GHCR.
