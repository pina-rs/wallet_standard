# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.4](https://github.com/pina-rs/wallet_standard/releases/tag/v0.7.4) (2026-10-26)

Grouped release for `core`.

### Features

- **Add subscription guards and wallet-following streams.** `Subscription` owns a listener closure and its unsubscribe function and detaches on drop, replacing the `on_register` / `on_unregister` API whose closure and dispose handle could drift apart into a leaked listener or a silently dead one. `BrowserWallet::account_changes` streams the wallet's account changes through `standard:events` so apps stop going stale when the user switches accounts inside the wallet, `BrowserWallet::on_change_with` exposes the raw change notifications, and `watch_wallets` streams the page's wallet registry including late registrations. The leptos example follows account changes through the stream. _Packages:_ 🟠 _wallet_standard_browser_ _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #81](https://github.com/pina-rs/wallet_standard/pull/81)

## [0.7.3](https://github.com/pina-rs/wallet_standard/releases/tag/v0.7.3) (2026-10-26)

Grouped release for `core`.

### Features

#### Cut JS boundary crossings and add perf gates

_Packages:_ 🟠 _wallet_standard_, 🟠 _wallet_standard_browser_

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #78](https://github.com/pina-rs/wallet_standard/pull/78)

`BrowserWalletInfoSnapshot` and `BrowserWalletAccountInfoSnapshot` read every JavaScript property once into plain Rust values, so repeated reads, comparisons, and hashing stay on the Rust side of the boundary, and `BrowserWallet::connected` checks the stored account without cloning the handle. `is_solana_standard_compatible` shares one feature-map read with the standard check instead of re-entering JavaScript.

`supported_transaction_versions` now accepts the standard's set — `legacy`, `0`, and `1` (v1 per SIMD-0296 / SIMD-0385) — and rejects values outside it instead of accepting any `u8`. `WalletError::is_connection_error` groups the four connection-state failures so apps match once instead of four times.

Counted boundary-crossing tests pin exact JavaScript read counts for the hot paths, and a `perf` CI job gates the wasm binary size of a linking probe against a committed budget.

## [0.7.2](https://github.com/pina-rs/wallet_standard/releases/tag/v0.7.2) (2026-10-16)

Grouped release for `core`.

### Fixes

#### Verify sign-in resources and fix broken doctests

_Packages:_ 🟢 _wallet_standard_, 🟢 _wallet_standard_browser_

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #69](https://github.com/pina-rs/wallet_standard/pull/69)

`verify_sign_in` now checks the `Resources:` block of a signed Sign In With Solana statement instead of skipping it, so a wallet that drops a requested resource can no longer pass verification. The signed message is read from the output once and shared by the text check and the signature check instead of crossing the wasm boundary twice, and `create_sign_in_message_text` propagates write failures like the rest of the function rather than `unwrap`ping.

Nine doctests that failed to compile are fixed: the readme's Sign In With Solana example imported `Keypair` from the wrong crate and referenced an undefined wallet, its error-handling example called an undefined function, and the seven value illustrations in `types.rs` are now marked `rust,ignore` like the other illustrative snippets. The empty version markers in the root readme are filled in so `mdt check` passes again.

Wallet discovery reads the JavaScript `features` object once per compatibility check instead of once per probed feature, and the dependency lockfile plus the pinned cargo tools (cargo-dylint 6.2.0, cargo-nextest 0.9.148, cargo-semver-checks 0.51.0, wasm-bindgen-cli 0.2.129) are refreshed to their latest releases.

### Notes

#### Classify pull request changes before they reach a release

_Packages:_ ⚪ _wallet_standard_, ⚪ _wallet_standard_browser_

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #49](https://github.com/pina-rs/wallet_standard/pull/49)

Add a `changeset-policy` workflow that runs two gates on every pull request.

The changeset policy fails a pull request when changed release-owned paths are not covered by a changeset and posts one remediation comment. It now derives the changed packages from git history with `from: origin/main`, so it also compares each attached changeset bump against the classified change type.

The change-classification step posts the compatibility evidence for the pull request: public API, dependency, and metadata changes with a proposed bump per package. The report separates the current pull request from everything accumulated since the last release, so a pull request that adds an API is not mistaken for one that breaks it.

Classification runs at `--detection-level semantic` against a cargo-semver-checks matrix covering the default feature set and all features, so a break behind `solana` or `browser` is proven rather than inferred from syntax.

## [0.7.1](https://github.com/pina-rs/wallet_standard/releases/tag/v0.7.1) (2026-09-29)

Grouped release for `core`.

### Features

#### Implement Wallet Standard features 1.5.0

_Packages:_ 🟠 _wallet_standard_, 🟠 _wallet_standard_browser_

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #59](https://github.com/pina-rs/wallet_standard/pull/59) · _Related issues:_ [#55](https://github.com/pina-rs/wallet_standard/issues/55)

Catch up to `@solana/wallet-standard-features@1.5.0`:

- `solana:signAndSendAllTransactions` (spec 1.3.0): sign and send a whole batch in one wallet prompt, with each transaction settling independently — a refused transaction arrives as a `WalletSettled::Rejected` element instead of failing the batch, so partial success is a first-class result.
- `solana:signOffchainMessage` (spec 1.4.0, SRFC-3): the wallet wraps the app's UTF-8 message in the canonical preamble (domain, version, sorted signers) and signs it, returning the exact preamble bytes so verifiers need no wallet SDK.

Both ship as core traits plus browser bridges with flat wire-format inputs (`Uint8Array` signer keys, numeric `messageVersion`), and are covered by the headless-Chrome boundary suite: the settled-batch parsing is driven end to end against a mock JS wallet, and the offchain input shape is pinned against the spec.

### Documentation

- **Enforce and write missing API documentation.** `missing_docs = "deny"` is now a workspace lint, and every public item that lacked documentation is documented — with the reasoning behind the item, not a restatement of its signature: when the error occurs, why the feature exists, and what an app should do with it. _Packages:_ ⚪ _wallet_standard_, ⚪ _wallet_standard_browser_ _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #57](https://github.com/pina-rs/wallet_standard/pull/57)
- **Single-source the documentation with mdt.** Shared documentation now lives once in `.templates/docs.t.md` and is consumed wherever it is rendered: both crate readmes, the mdBook pages, the root readme, and rustdoc comments (`mdt update` syncs, `mdt check` fails CI on drift, both wired into `lint:all`). Dependency snippets now interpolate the workspace version, ending the readme/docs version drift (`0.5`/`0.5.1` → current), and the docs book's drifted API examples were replaced with the canonical, compiled-against implementations. _Packages:_ ⚪ _wallet_standard_, ⚪ _wallet_standard_browser_ _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #58](https://github.com/pina-rs/wallet_standard/pull/58)

### Notes

#### Sync single-sourced docs during the release commit

_Packages:_ ⚪ _wallet_standard_, ⚪ _wallet_standard_browser_

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #62](https://github.com/pina-rs/wallet_standard/pull/62) · _Related issues:_ [#56](https://github.com/pina-rs/wallet_standard/issues/56), [#61](https://github.com/pina-rs/wallet_standard/issues/61)

The release commit rewrites `[workspace.package.version]`, and the `install_deps` provider interpolates that version into both crate readmes and the getting-started page. Nothing re-synced those copies, so the release commit invalidated its own consumers and `lint:docs` failed the release pull request with three stale blocks — after the version bump had already been committed.

The `monochange` release flow now runs `fix:docs` (`mdt update`) between `PrepareRelease` and the format step, so the release commit carries the version bump and every generated copy of it together. Keeping them in one commit matters: a follow-up commit would leave the release tag pointing at a revision whose readmes still advertised the previous version.

`mdt` also moves from a `curl`-installed tarball in CI to the devenv profile, matching how `monochange` is provisioned. CI and the release flow now run the same pinned tool, so a consumer block can never be generated by one version and checked by another.

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
