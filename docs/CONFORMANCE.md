# Conformance & HelixTest

Lab Kit **does not embed** the HelixTest suite; it **invokes** the separate repository [SynapticFour/HelixTest](https://github.com/SynapticFour/HelixTest) as a tool (same pattern as Ferrum).
HelixTest usage here is GA4GH-centric. Optional MII/KDS checks run separately via `lab-kit mii ...` (delegated to Ferrum MII Connect).

**Pinned revision for CI:** `config/ci/helixtest-revision.txt` — currently **`4a10e126c219`** (Ferrum `HELIXTEST_SHA`, tag label v0.1.2). Bump it in a dedicated change when upgrading HelixTest. GitHub Actions workflow **Conformance** checks out that revision, builds `helixtest-cli`, and runs a CLI smoke test (no live services). A full `--all` run against Compose is **opt-in** via **workflow_dispatch** with `run_live_suite=true`, and also runs automatically when `config/ci/ferrum-revision.txt` is pushed (Ferrum pin bump). When that live step runs, HelixTest failures **fail the job** — they are not masked. A green compose-profile job is **not** certification.

## Run

```bash
# After `docker compose up` with a valid profile:
export HELIXTEST_BIN=helixtest   # or path to the HelixTest CLI
lab-kit conformance run --config lab-kit.toml
```

`lab-kit conformance run` always passes HelixTest **`--all --mode ferrum --report json`** plus `--only` for enabled surfaces. Invoking bare `helixtest` prints “Nothing to do” and exits 0 — that is **not** treated as a pass. The runner kills the process on timeout.

## Subset profiles (companion story)

Lab Kit is one, two, or three Ferrum surfaces — not a second monolith. After compose is up:

```bash
# Beacon-only profile → HelixTest Beacon
lab-kit init --profile beacon-only --non-interactive
# … generate compose, docker compose up …
helixtest --all --mode ferrum --report json --only beacon

# DRS + WES profile
lab-kit init --profile drs-wes --non-interactive
helixtest --all --mode ferrum --report json --only drs --only wes
```

CI generates `beacon-only`, `field-edge`, and `federated-node` compose on every relevant push. A live HelixTest run against those stacks is **opt-in** (`workflow_dispatch` + `run_live_suite=true`).

Live **ferrum+infra** (Passport-on-DRS) is Ferrum’s scheduled job; Lab Kit’s federated-node profile generates the matching surfaces. See [FEDERATED-NODE.md](FEDERATED-NODE.md).

HelixTest should emit JSON results (format may vary; `lab-kit-report` accepts flexible shapes).

## Reports

```bash
lab-kit conformance report \
  --helixtest-json path/to/helixtest-output.json \
  --out-dir reports/conformance \
  --config lab-kit.toml
```

Outputs:

- **`conformance-report.json`** — always written (machine-readable). **Empty JSON or skip-only rows fail** (`overall_pass = false`).
- **`conformance-report.pdf`** — written only if **`FERRUM_LAB_KIT_LICENSE_KEY`** is a **signed** `flk1.<payload>.<sig>` token that verifies against the operator public key and matches a prior **`lab-kit license activate`**. Unparseable `expires_at` is **fail-closed**. This **does not** gate GA4GH compliance; only the PDF artifact.

## Reading the report

- **Per-service table:** pass/fail per enabled GA4GH surface.
- **Overall summary:** aggregate pass/fail (never true when no tests executed).
- **Next steps:** remediation hints for failed checks (attach to grant / ELIXIR node packages).
