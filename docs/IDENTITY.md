# Who Ferrum Lab Kit is for

Ferrum Lab Kit **comes with Ferrum**. It is a Ferrum companion, not a fifth product. It generates Compose / Helm / systemd so a lab can turn on **one, two, or three** GA4GH surfaces (`FERRUM_SERVICES__ENABLE_*`) around the SHA-pinned Ferrum image.

It does **not** implement GA4GH protocol logic.

## Audience

Ferrum operators who do not want the full monolith surface on day one.

**Not for:** a second GA4GH server, a product without Ferrum.

## Standalone (still needs Ferrum)

```bash
git clone https://github.com/SynapticFour/Ferrum-Lab-Kit.git && cd Ferrum-Lab-Kit
make prove
lab-kit init --profile field-edge --non-interactive
# Archive bundles (no WES/TES, Metadata Store on):
# lab-kit init --profile archive-submitter --non-interactive
# Technical federated node (not EGA membership):
# lab-kit init --profile federated-node --non-interactive
```

HelixTest pin for CI: tag **v0.1.3** / SHA **`1832c043e167`** (Ferrum `VERSIONS.lock` `HELIXTEST_SHA`). Ferrum image/git pin: **v0.3.2** (`2bd147c9…`) — same SHA as `config/ci/ferrum-revision.txt` / `ferrum-image.txt`. Lab Kit crate `0.1.0` is the companion train, not a second product version.

Subset proof: `beacon-only` + `helixtest --only beacon`, `drs-wes` + `--only drs --only wes`. See [CONFORMANCE.md](CONFORMANCE.md).

Optional **BRA workbench companion** (same pattern as Solum): profile `bra-companion` / `--with-bra`. Lab Kit only sets `FERRUM_DRS_URL` / `FERRUM_WES_URL`. Bring `BRA_IMAGE`. See [BRA-CO-DEPLOY.md](BRA-CO-DEPLOY.md).
