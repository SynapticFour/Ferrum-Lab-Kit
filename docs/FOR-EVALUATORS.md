# For evaluators

A factual page for someone checking this repository. Not a sales brief. Not legal advice.

## Status

**Early use.** Registry: TOOL-LAB-KIT-STATUS-001. The older README line said "Early access". That is the same stage, not a second one. Early use is not an official seal.

The workflow file (`.github/workflows/ci.yml`) on the main branch runs `cargo test --workspace`, then generates Compose for `field-edge`, `bra-companion`, and `federated-node`, and checks the YAML. This page does not record a run of that workflow.

## What this repository is

It writes Compose, Helm, and systemd around a pinned Ferrum image. It does not implement the GA4GH interfaces. Those stay in [Ferrum](https://github.com/SynapticFour/Ferrum). Registry: TOOL-LAB-KIT-USE-001.

## Pin

[`config/ci/ferrum-revision.txt`](../config/ci/ferrum-revision.txt) pins Ferrum tag **v0.3.2** at `2bd147c99735866ac9cfb530f62ef984c8efce8b`. That pin is deliberate. It is the file on this commit, not a promise that a later commit keeps the same tag. Ferrum's default branch is newer. The pin is not "current Ferrum is v0.3.2".

## Licence

Ferrum Lab Kit is free under the Apache License, Version 2.0. Paid are services and commitments. Prices are on the [consulting page](https://synapticfour.com/en/consulting/). The pinned Ferrum version was released under BUSL-1.1; see Ferrum's [LICENSING-HISTORY.md](https://github.com/SynapticFour/Ferrum/blob/main/LICENSING-HISTORY.md). Versions of this repository released before this change were published under the Business Source License 1.1; see [LICENSING-HISTORY.md](../LICENSING-HISTORY.md). The file is [LICENSE](../LICENSE).

## What this page does not record

- A run of `make up` or of the Pi installer.
- A clinical pilot.
- GA4GH certification. A HelixTest report from `lab-kit conformance`, when one is produced, is a technical signal.

Contact: [contact@synapticfour.com](mailto:contact@synapticfour.com)
