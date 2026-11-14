# Business model

The software is free under the Apache License, Version 2.0. See [LICENSE](../LICENSE) and [LICENSING-HISTORY.md](../LICENSING-HISTORY.md). Paid are services and commitments. Prices are on the [consulting page](https://synapticfour.com/en/consulting/).

The pinned Ferrum version (v0.3.2) was released under BUSL-1.1. See Ferrum's [LICENSING-HISTORY.md](https://github.com/SynapticFour/Ferrum/blob/main/LICENSING-HISTORY.md). This repository does not change that pin.

This page is not legal advice and not a price list.

## What the code does

- It generates Compose, Helm, and systemd around the pinned Ferrum image.
- JSON conformance output is not behind a key.
- PDF conformance output checks a signed `FERRUM_LAB_KIT_LICENSE_KEY` (`flk1.<payload>.<sig>`) and `lab-kit license activate`, because that is what `lab-kit-report` does. The key does not gate running a GA4GH stack.

Lab Kit comes with Ferrum. It is not a separate product you buy a copyright licence for. Solum stays under the Business Source License 1.1 in its own repository.

Contact: [contact@synapticfour.com](mailto:contact@synapticfour.com).
