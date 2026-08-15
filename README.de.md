# Ferrum Lab Kit

**Ferrum Lab Kit** ist ein **Compose-/Helm-/systemd-On-Ramp** zu [Ferrum](https://github.com/SynapticFour/Ferrum): es erzeugt Deploy-YAML, Env-Dateien und Operator-Tools um das **Ferrum-Monolith-Image** (`ghcr.io/synapticfour/ferrum`, SHA-Pin). Es ist ein **eigenes Repository** (kein Fork) und **implementiert keine GA4GH-Protokoll-Logik**. Runtime-I/O (Beacon/DRS/WES/…) ist der Ferrum-Container.

Adapter-Traits (`lab-kit-adapters`) und OIDC-Helfer (`lab-kit-auth`) sind **Lab-Kit-Bibliotheken**. Prüfung: `lab-kit adapters check`. **Ferrum hängt zur Laufzeit nicht von diesen Crates ab.**

## GA4GH-Stack

Lab Kit ist der **Deploy-On-Ramp**. Übersicht: **[docs/ECOSYSTEM.md](docs/ECOSYSTEM.md)** (Ferrum, ga4gh-infra, Demo, HelixTest).

## CLI installieren (optional)

```bash
./install.sh              # Release-Build → target/release/lab-kit
./install.sh --install    # zusätzlich cargo install (Standard: ~/.cargo/bin)
./install.sh --install --prefix "$HOME/.local"   # → ~/.local/bin
```

Benötigt [Rust](https://rustup.rs); `rust-toolchain.toml` im Repo wird berücksichtigt.

## Kurzweg: Beacon v2 + DRS (ca. 5 Befehle)

```bash
git clone https://github.com/SynapticFour/Ferrum-Lab-Kit.git && cd Ferrum-Lab-Kit
cp .env.example .env
lab-kit init --profile field-edge --non-interactive
lab-kit generate compose --config lab-kit.toml --fragments deploy/docker-compose -o docker-compose.yml
docker compose -f docker-compose.yml up -d
# Gateway: http://127.0.0.1:8080/health
```

Oder einmalig: `./install-edge.sh` / `make up`.

### Raspberry Pi Kit

```bash
lab-kit generate raspberry-pi --output ./pi-kit   # oder: make pi-kit
# Auf dem Pi: cd pi-kit && ./install-on-pi.sh
lab-kit generate pi --with-solum --ram-gb 8 -o ./pi-kit
```

Details: [docs/RASPBERRY-PI.md](docs/RASPBERRY-PI.md).

**Selektion:** Profile/`lab-kit.toml` wählen Surfaces; Compose startet den **Monolith** `ghcr.io/synapticfour/ferrum` und setzt `FERRUM_SERVICES__ENABLE_*`. Details: [docs/FERRUM-INTEGRATION.md](docs/FERRUM-INTEGRATION.md).

## Co-Deploy mit ga4gh-infra

```bash
./install-edge.sh --with-infra
# oder: make up-with-infra
```

Passport-Broker + Service-Registry neben Ferrum (Ports 8180–8190, mock-idp 9100).

## Optional: Solum-Begleiter

Consent-Sidecar (Ferrum pollt Solum) — Produkt bleibt in Solum:

```bash
./install-edge.sh --with-solum
# oder: make up-with-solum / make up-with-infra-solum
```

Siehe [docs/SOLUM-CO-DEPLOY.md](docs/SOLUM-CO-DEPLOY.md).

### Lokaler Lifecycle (Make)

| Ziel | Befehl |
|------|--------|
| Start | `make up` |
| + ga4gh-infra | `make up-with-infra` |
| + Solum | `make up-with-solum` |
| + beide | `make up-with-infra-solum` |
| Stop (Daten behalten) | `make down` |
| Volumes löschen | `make destroy` |

## Dienstauswahl

| GA4GH-Oberfläche | Nutzen (Beispiele) |
|------------------|--------------------|
| **Beacon v2** | ELIXIR Beacon Network, öffentliche/registrierte/kontrollierte Kohorten-Metadaten |
| **DRS** | Stabile Datenobjekt-IDs über S3/POSIX |
| **htsget** | Effizientes Streaming genomischer Daten |
| **WES / TES** | Portierbare Workflows und Task-Ausführung auf **SLURM**/K8s |
| **TRS** | Tool-/Workflow-Registry (z. B. nf-core) |

Mehr in [docs/GA4GH-STANDARDS.md](docs/GA4GH-STANDARDS.md).

## Dokumentation (Überblick)

- **[docs/README.md](docs/README.md)** — Index aller Guides und Beispiele
- [GA4GH-Workflow-Primer](docs/GA4GH-WORKFLOW-PRIMER.md) — Ablauf TRS/WES/TES, DRS, Engines
- [Operations-Checkliste](docs/OPERATIONS-CHECKLIST.md) — Env-Vars, Docker, Netzwerk
- [Solum-Co-Deploy](docs/SOLUM-CO-DEPLOY.md) — optionaler Consent-Begleiter
- [Raspberry Pi](docs/RASPBERRY-PI.md) — Field-Kit + Installation auf dem Gerät
- [Ferrum-Integration](docs/FERRUM-INTEGRATION.md) · [Deployment](docs/DEPLOYMENT-TARGETS.md) · [ELIXIR AAI](docs/ELIXIR-AAI.md)

## Für wen ist das gedacht?

- Universitäts- und Institutslabore in **Deutschland, Österreich, der Schweiz** — typischerweise **SLURM**-Cluster oder Einzelserver.
- **ELIXIR-Node**-Kandidaten mit dokumentierter, konformer Teilmenge.
- **GDI**-Nationalknoten, **Seltene-Erkrankungen**-Konsortien und **NFDI**-nahe Projekte.
- **Field Labs** (Pi/Laptop) — siehe [Field/Edge](docs/DEPLOYMENT-TARGETS.md#field-edge).

## Open-Core-Modell

**GA4GH-Deploy-Tooling** steht unter **BUSL-1.1** (siehe [LICENSE](LICENSE)) — BUSL ist **kein** OSI-Open-Source-Lizenztext; nicht-kommerzielle Forschung ist über den Additional Use Grant erlaubt. **PDF-Konformitätsberichte** brauchen einen **signierten** `FERRUM_LAB_KIT_LICENSE_KEY` (`flk1.<payload>.<sig>`) und **`lab-kit license activate`**. JSON-Berichte und der GA4GH-Stack sind nicht lizenzgeschützt. Details: [docs/BUSINESS-MODEL.md](docs/BUSINESS-MODEL.md).

## CLI (`lab-kit`)

| Befehl | Zweck |
|--------|--------|
| `lab-kit init` | Interaktiver Wizard → `lab-kit.toml` |
| `lab-kit generate compose` | Compose (Monolith + optional Infra/Solum) |
| `lab-kit generate compose --with-ga4gh-infra` | ga4gh-infra erzwingen |
| `lab-kit generate compose --with-solum` | Solum-Sidecar erzwingen |
| `lab-kit generate raspberry-pi` / `pi` | Portables Pi-Field-Kit |
| `lab-kit generate helm` / `systemd` | Deploy-Artefakte |
| `lab-kit generate infra-secrets` | RSA-PEMs + `secrets.env` für ga4gh-infra (gitignored) |
| `lab-kit adapters check` | POSIX/SQLite lokal prüfen; SLURM/S3/Nextflow nur berichten |
| `lab-kit status` | Health der konfigurierten Dienste |
| `lab-kit conformance run` / `report` | HelixTest mit `--all` (leer/skip-only ≠ Pass) + Berichte |
| `lab-kit ferrum check` | Git-gepinnter `ferrum-core`-Link prüfen |
| `lab-kit ingest …` | Ferrum **`/api/v1/ingest/*`** — siehe [Ferrum INGEST-LAB-KIT](https://github.com/SynapticFour/Ferrum/blob/main/docs/INGEST-LAB-KIT.md) |
| `lab-kit mii …` | Optionaler Passthrough zu `ferrum mii …` |

## Vollplattform

Wer die komplette souveräne Plattform braucht: **[github.com/SynapticFour/Ferrum](https://github.com/SynapticFour/Ferrum)**.

## English README

See [README.md](README.md).

## Mitwirken

Siehe [CONTRIBUTING.md](CONTRIBUTING.md).
