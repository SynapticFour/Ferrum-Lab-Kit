# Conformance & HelixTest

Lab Kit **does not embed** the HelixTest suite; it **invokes** the separate repository [SynapticFour/HelixTest](https://github.com/SynapticFour/HelixTest) as a tool (same pattern as Ferrum).

**Pinned revision for CI:** `config/ci/helixtest-revision.txt` (full Git SHA). Bump it in a dedicated PR when upgrading HelixTest. GitHub Actions workflow **Conformance** checks out that revision, builds `helixtest-cli`, and runs a CLI smoke test. A full `--all` run against Compose is available only via **workflow_dispatch** with `run_live_suite` (needs working Ferrum images).

## Run

```bash
# After `docker compose up` with a valid profile:
export HELIXTEST_BIN=helixtest   # or path to the HelixTest CLI
lab-kit conformance run --config lab-kit.toml
```

HelixTest should emit JSON results (format may vary; `lab-kit-report` accepts flexible shapes).

## Reports

```bash
lab-kit conformance report \
  --helixtest-json path/to/helixtest-output.json \
  --out-dir reports/conformance \
  --config lab-kit.toml
```

Outputs:

- **`conformance-report.json`** — always written (machine-readable, suitable for APIs and archives).
- **`conformance-report.pdf`** — written only if **`FERRUM_LAB_KIT_LICENSE_KEY`** is set to a non-empty value (commercial tier). This **does not** gate GA4GH compliance; only the PDF artifact.

## Reading the report

- **Per-service table:** pass/fail per enabled GA4GH surface.
- **Overall summary:** aggregate pass/fail.
- **Next steps:** remediation hints for failed checks (attach to grant / ELIXIR node packages).
