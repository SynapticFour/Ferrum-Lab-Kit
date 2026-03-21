# Ferrum integration

Lab Kit depends on the **[Ferrum](https://github.com/SynapticFour/Ferrum)** platform as **library code**, not as a fork.

## Git pin (`ferrum-core`)

- Crate: **`lab-kit-ferrum`** → `ferrum-core` from  
  `https://github.com/SynapticFour/Ferrum.git` pinned by **full git `rev`** (see `crates/lab-kit-ferrum/Cargo.toml`).
- **Bump procedure:** pick a Ferrum commit (often `main` HEAD), set the same SHA in `Cargo.toml` and **`config/ci/ferrum-revision.txt`**, then `cargo update -p ferrum-core` and run tests.

## CLI check

```bash
cargo run -p lab-kit-selector -- ferrum check
```

Prints the linked `ferrum_core::FerrumError` type name and the pinned revision.

## Runtime wiring

Deploy generators still use **placeholder images** until you point Compose/Helm at Ferrum release images or build from the Ferrum repo. Shared **types, config, and auth** primitives come from `ferrum-core` via `lab-kit-ferrum` for gateways and future glue code.
