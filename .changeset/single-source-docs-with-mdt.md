---
wallet_standard: docs
wallet_standard_browser: docs
---

# Single-source the documentation with mdt

Shared documentation now lives once in `.templates/docs.t.md` and is consumed wherever it is rendered: both crate readmes, the mdBook pages, the root readme, and rustdoc comments (`mdt update` syncs, `mdt check` fails CI on drift, both wired into `lint:all`). Dependency snippets now interpolate the workspace version, ending the readme/docs version drift (`0.5`/`0.5.1` → current), and the docs book's drifted API examples were replaced with the canonical, compiled-against implementations.
