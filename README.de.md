# Ferrum Lab Kit

**Ferrum Lab Kit** ist die **Einstiegslösung (On-Ramp)** zu [Ferrum](https://github.com/SynapticFour/Ferrum): eine **Deployments- und Integrationsschicht** für kleine und mittlere Forschungslabore, **ELIXIR-Knoten-Kandidaten**, **GHGA**-Datensubmitting und **GDI**-Teilnehmer, die **ausgewählte GA4GH-konforme Dienste** betreiben wollen, ohne die gesamte Ferrum-Plattform auszurollen. Es ist ein **eigenes Repository** (kein Fork) und **implementiert keine GA4GH-Logik erneut**; es konfiguriert Ferrum-Komponenten gegen **Ihre** Storage-, Scheduler- und Identity-Infrastruktur.

## CLI installieren (optional)

```bash
./install.sh              # Release-Build → target/release/lab-kit
./install.sh --install    # zusätzlich cargo install (Standard: ~/.cargo/bin)
./install.sh --install --prefix "$HOME/.local"   # → ~/.local/bin
```

Benötigt [Rust](https://rustup.rs); `rust-toolchain.toml` im Repo wird berücksichtigt.

## Kurzweg: Beacon v2 + ELIXIR LS Login (ca. 5 Befehle)

```bash
git clone https://github.com/SynapticFour/Ferrum-Lab-Kit.git && cd Ferrum-Lab-Kit
cp config/profiles/beacon-only.toml lab-kit.toml
# lab-kit.toml bearbeiten: echte [auth.ls-login] client_id / client_secret setzen
cargo run -p lab-kit-selector -- generate compose --config lab-kit.toml --fragments deploy/docker-compose --output docker-compose.yml
docker compose -f docker-compose.yml up -d
# Sobald Ferrum-Images verfügbar sind — bis dahin sind Compose-Images Platzhalter.
```

## Dienstauswahl

| GA4GH-Oberfläche | Nutzen (Beispiele) |
|------------------|--------------------|
| **Beacon v2** | ELIXIR Beacon Network, öffentliche/registrierte/kontrollierte Kohorten-Metadaten |
| **DRS** | Stabile Datenobjekt-IDs über S3/POSIX |
| **htsget** | Effizientes Streaming genomischer Daten |
| **WES / TES** | Portierbare Workflows und Task-Ausführung auf **SLURM**/K8s |
| **TRS** | Tool-/Workflow-Registry (z. B. nf-core) |

Mehr in [docs/GA4GH-STANDARDS.md](docs/GA4GH-STANDARDS.md).

## Für wen ist das gedacht?

- Universitäts- und Institutslabore in **Deutschland, Österreich, der Schweiz** — typischerweise **SLURM**-clusters oder Einzelserver.
- **ELIXIR-Node**-Kandidaten mit dokumentierter, konformer Teilmenge.
- **GDI**-Nationalknoten, **Seltene-Erkrankungen**-Konsortien und **NFDI**-nahe Projekte, die Nachweise für Anträge benötigen.

## Open-Core-Modell

**GA4GH-Deployments und LS-Login-Integration** stehen unter **BUSL-1.1** (siehe [LICENSE](LICENSE)) für zulässige **nicht-kommerzielle Forschung**. **PDF-Konformitätsberichte** und Enterprise-Föderations-Features sind **kommerziell**; die PDF-Ausgabe prüft **`FERRUM_LAB_KIT_LICENSE_KEY`** — **JSON-Berichte und die Protokoll-Stacks selbst sind nicht lizenzgeschützt.** Details: [docs/BUSINESS-MODEL.md](docs/BUSINESS-MODEL.md).

## Kontext: de.NBI, GHGA, NFDI

- **de.NBI / NFDI:** Standards-basierte, nachnutzbare Dienste statt Insellösungen.  
- **GHGA:** Datenhaltung und eingereichte Datensätze mit GA4GH-konformen Schnittstellen verknüpfbar.  
- **NFDI-Konsortien:** Modulare Integration bestehender HPC- und Storage-Landschaften.

## Vollplattform

Wer die komplette souveräne Plattform braucht: **[github.com/SynapticFour/Ferrum](https://github.com/SynapticFour/Ferrum)**.

## CLI (`lab-kit`)

| Befehl | Zweck |
|--------|--------|
| `lab-kit init` | Interaktiver Wizard → `lab-kit.toml` |
| `lab-kit generate compose` / `helm` / `systemd` | Deploy-Artefakte erzeugen |
| `lab-kit status` | Health der konfigurierten Dienste |
| `lab-kit conformance run` / `report` | HelixTest + Berichte |
| `lab-kit ferrum check` | Git-gepinnter `ferrum-core`-Link prüfen |
| `lab-kit ingest …` | Aufruf von Ferrum **`/api/v1/ingest/*`** (Maschinen-Ingest) — siehe [Ferrum `docs/INGEST-LAB-KIT.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/INGEST-LAB-KIT.md) und [docs/FERRUM-INTEGRATION.md](docs/FERRUM-INTEGRATION.md) |

## English README

See [README.md](README.md).

## Mitwirken

Siehe [CONTRIBUTING.md](CONTRIBUTING.md).
