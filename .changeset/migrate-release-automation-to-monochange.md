---
wallet_standard: none
wallet_standard_browser: none
---

# Migrate release automation to monochange

Replace knope with monochange: a release PR flow on every push to main, crates.io trusted publishing through the `publisher` environment, and `release:local` / `publish:local` devenv scripts as a manual fallback.
