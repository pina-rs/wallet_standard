# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0](https://github.com/pina-rs/wallet_standard/releases/tag/v0.7.0) (2026-09-28)

Grouped release for `core`.

### Breaking Changes

#### Fix browser wallet interop and panic safety

_Packages:_ 🟢 _wallet_standard_, 🔴 _wallet_standard_browser_

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

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #55](https://github.com/pina-rs/wallet_standard/pull/55) · _Related issues:_ [#45](https://github.com/pina-rs/wallet_standard/issues/45)

## [0.6.0](https://github.com/pina-rs/wallet_standard/releases/tag/v0.6.0) (2026-09-07)

Grouped release for `core`.

### Breaking Changes

#### Update Solana dependencies to Agave 4.x

_Packages:_ _wallet_standard_, _wallet_standard_browser_

Update the Solana dependencies to the latest Agave 4.x crates, raise the MSRV to 1.89.0, and unify both crates on a single workspace version.

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #44](https://github.com/pina-rs/wallet_standard/pull/44)

### Notes

#### Migrate release automation to monochange

_Packages:_ _wallet_standard_, _wallet_standard_browser_

Replace knope with monochange: a release PR flow on every push to main, crates.io trusted publishing through the `publisher` environment, and `release:local` / `publish:local` devenv scripts as a manual fallback.

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #44](https://github.com/pina-rs/wallet_standard/pull/44)
