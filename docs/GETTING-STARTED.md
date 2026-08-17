# Getting started

Ferrum Lab Kit wraps the **Ferrum monolith image**. It does not implement DRS/WES/TES itself.

## Compose (shortest)

```bash
git clone https://github.com/SynapticFour/Ferrum-Lab-Kit.git && cd Ferrum-Lab-Kit
make up
```

Equivalent: `./install-edge.sh`. Gateway: `http://127.0.0.1:8080/health`.

Optional companions:

```bash
make up-with-infra
make up-with-solum
make up-with-infra-solum
```

Stop: `make down`. Remove volumes: `make destroy`.

## CLI (optional)

Needs [Rust](https://rustup.rs):

```bash
./install.sh              # release build → target/release/lab-kit
./install.sh --install    # cargo install (default: ~/.cargo/bin)
```

Then, for a Beacon v2 + DRS edge:

```bash
cp .env.example .env
lab-kit init --profile field-edge --non-interactive
lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose -o docker-compose.yml
docker compose -f docker-compose.yml up -d
```

Raspberry Pi: `make pi-kit` then on the device `cd pi-kit && ./install-on-pi.sh`. Guide: [RASPBERRY-PI.md](RASPBERRY-PI.md).

Optional BRA workbench (bring `BRA_IMAGE`; not a combo SKU): `lab-kit init --profile bra-companion --non-interactive` then `lab-kit generate compose --with-bra`. See [BRA-CO-DEPLOY.md](BRA-CO-DEPLOY.md).

Profiles set `FERRUM_SERVICES__ENABLE_*` on a named Ferrum image variant (`-edge`, `-edge-infra`, or full). Details: [FERRUM-INTEGRATION.md](FERRUM-INTEGRATION.md).
