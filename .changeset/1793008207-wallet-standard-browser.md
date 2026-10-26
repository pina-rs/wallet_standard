---
wallet_standard_browser: feat
---

# Add one-pass metadata snapshots, single-read feature-map compatibility checks, and standard-set transaction version validation

BrowserWalletInfoSnapshot and BrowserWalletAccountInfoSnapshot read each JS property once; is_solana_standard_compatible shares one features read with the standard check; supported_transaction_versions accepts legacy/0/1 and rejects values outside the standard set; a wasm size gate and counted boundary-crossing tests pin the hot paths in CI.
