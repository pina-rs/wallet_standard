---
wallet_standard: fix
wallet_standard_browser: breaking
---

# Fix browser wallet interop and panic safety

`wallet_standard_browser` 0.6.0 shipped several defects that made the browser integration unusable against real Wallet Standard wallets. The transaction signing APIs had never successfully executed against a JavaScript wallet, so their input shape is corrected as a breaking release rather than papered over:

- **Breaking**: `SolanaSignTransactionInput` and `SolanaSignAndSendTransactionInput` are flat wire-format structs (`account`, `transaction` bytes, `chain`, `options`) matching `@solana/wallet-standard-features`, replacing the `props`-flattened shapes that `serde-wasm-bindgen` serialized as empty objects.
- The `solana` feature did not enable the `solana-transaction` optional dependency, so `wallet_standard_browser` with only `features = ["solana"]` failed to compile outside of `--all-features` builds.
- `WalletStandardConnect::connect_with_options` and `WalletStandardDisconnect::disconnect` on `BrowserWallet` called themselves instead of delegating to the registered wallet's feature, causing infinite recursion (a WASM stack overflow at runtime).
- `sign_transactions` serialized the `WalletResult` wrapper instead of the collected inputs, so JS wallets received `{"Ok":[{}]}` rather than the input array.
- `BrowserWalletAccountInfo::try_new` performed an `instanceof` check against the `#[wasm_bindgen(module = ...)]` extern, emitting a named import that `/js/app.js` does not export and breaking module instantiation for any bundle that called it.
- The `solana:signAndSendTransaction` feature read `supported_transaction_versions` (snake case) instead of `supportedTransactionVersions`, breaking every JS wallet.
- The `experimental:encrypt` output read `cipher_text` instead of the spec's `ciphertext` (`@wallet-standard/experimental-features`), and the decrypt props serialized the same field as `cipherText`.
- JS rejections converted through `WalletError::from(JsValue)` now surface the `message` of `Error`/`DOMException` values instead of a fixed placeholder.
- `Wallets::try_register` registers without panicking; the page-controlled registry can no longer take down the WASM module through `Wallets::register`.

CI now covers what let these ship: a `check-features` job builds every feature combination a downstream user can name (including `wasm32` targets), a `test-browser` job runs the rewritten `wasm-bindgen-test` suite in headless Chrome — including boundary-shape regression tests for every input struct — and an `e2e` job runs the new Leptos 0.8 / Dioxus 0.7 examples plus Playwright suite against a local surfpool node (`examples/e2e`).
