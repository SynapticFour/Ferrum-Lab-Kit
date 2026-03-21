# Optional Traefik sidecar (generic)

Lab Kit does **not** assume a specific Traefik Helm chart or Compose stack. If you do not already run Traefik, this folder is a **minimal, copy-paste-friendly** example:

- **Static** Traefik container (Docker Compose)
- **Dynamic** config from a host directory (`./dynamic/`), where you drop the file Lab Kit generates: `proxy-traefik-dynamic.yaml`

## Steps

1. Generate compose + proxy config from your `lab-kit.toml` (with at least one `external_url`):

   ```bash
   lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose --output generated/docker-compose.yml
   ```

2. Copy the generated dynamic file next to this README:

   ```bash
   mkdir -p deploy/traefik/dynamic
   cp generated/proxy-traefik-dynamic.yaml deploy/traefik/dynamic/
   ```

3. Start Traefik from **this directory**:

   ```bash
   cd deploy/traefik
   docker compose up -d
   ```

4. Send traffic to **port 9080** on the host (see `docker-compose.yml`). Put a real hostname / TLS in front when you move to production (e.g. institute load balancer, or extend this Compose with certificates).

## Production notes

- Replace host port `9080` with `80`/`443` and add TLS (Let’s Encrypt ACME, or your institute terminates TLS upstream).
- If you already have Traefik/Kubernetes Ingress, **ignore** this folder and point your existing Traefik **file provider** or **IngressRoute** CRDs at `proxy-traefik-dynamic.yaml` (same YAML structure).

Path prefixes in `proxy-traefik-dynamic.yaml` follow **[Ferrum](https://github.com/SynapticFour/Ferrum) `ferrum-gateway`** (`/ga4gh/...`, `/passports/v1`, …).
