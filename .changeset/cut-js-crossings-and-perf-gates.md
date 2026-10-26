---
wallet_standard: minor
wallet_standard_browser: minor
---

# Cut JS boundary crossings and add perf gates

`BrowserWalletInfoSnapshot` and `BrowserWalletAccountInfoSnapshot` read every JavaScript property once into plain Rust values, so repeated reads, comparisons, and hashing stay on the Rust side of the boundary, and `BrowserWallet::connected` checks the stored account without cloning the handle. `is_solana_standard_compatible` shares one feature-map read with the standard check instead of re-entering JavaScript.

`supported_transaction_versions` now accepts the standard's set — `legacy`, `0`, and `1` (v1 per SIMD-0296 / SIMD-0385) — and rejects values outside it instead of accepting any `u8`. `WalletError::is_connection_error` groups the four connection-state failures so apps match once instead of four times.

Counted boundary-crossing tests pin exact JavaScript read counts for the hot paths, and a `perf` CI job gates the wasm binary size of a linking probe against a committed budget.
