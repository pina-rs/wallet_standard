# Browser Integration

`wallet_standard_browser` runs inside a WASM module in the browser and bridges the JavaScript Wallet Standard both ways:

- **inbound**: JavaScript wallets registered on the page become Rust `BrowserWallet` values.
- **outbound**: Rust wallet implementations become JavaScript wallets registered on the page.

## Discovering wallets

```rust,ignore
use wallet_standard_browser::prelude::*;

let wallets = get_wallets();
let phantom = wallets
    .get()
    .into_iter()
    .find(|wallet| wallet.name() == "Phantom")
    .ok_or(WalletError::UnsupportedFeature {
        feature: "standard:connect".into(),
        wallet: "Phantom".into(),
    })?;

let mut wallet = BrowserWallet::builder().wallet(phantom).build();
let accounts = wallet.connect().await?;
let account = accounts.first().cloned().ok_or(WalletError::WalletConnection)?;
```

`get_wallets()` returns a snapshot of every wallet registered on the page at call time. Each returned `BrowserWallet` implements `Wallet`, `WalletStandardConnect`, `WalletStandardDisconnect` and (with the `solana` feature) the full Solana feature set — including `WalletSolanaSignMessage`, `WalletSolanaSignTransaction`, `WalletSolanaSignAndSendTransaction` and `WalletSolanaSignIn`.

## Registering a Rust wallet into the page

Wallet-side registration requires constructing the JS wallet object — including feature objects whose methods are Rust closures exposed to JS — and then registering it. A complete, working implementation (connect, disconnect, events, `solana:signMessage`, `solana:signTransaction`, `solana:signAndSendTransaction`) lives in `examples/crates/surfpool-wallet-core/src/wallet.rs` and is exercised by the Leptos and Dioxus examples under `examples/`:

```rust,ignore
use wallet_standard_browser::prelude::*;

// Build a wallet object whose `features` map holds feature objects with
// Rust-backed JS callbacks (see the example for the full construction).
let wallet = build_wallet_object(/* … */);

// After this call, every @wallet-standard/app consumer on the
// page (React apps, other extensions) sees your wallet.
register_wallet(wallet)?;
```

The browser bridge bundles the official `@wallet-standard/app` and `@wallet-standard/wallet` ESM modules under `crates/wallet_standard_browser/js/` and binds them with `wasm-bindgen`. The `copy:js` devenv script refreshes those bundles.

## Events

Wallets that support `standard:events` expose change notifications. The browser bridge converts the JS event system into the `ConnectedWalletStandardEvents` trait, so apps subscribe with the same Rust API regardless of which wallet emitted the event.

## Serialization at the boundary

Props and outputs cross the JS boundary through:

- `serde-wasm-bindgen` for JSON-shaped data (wallet info, inputs),
- `bincode` for transaction bytes,
- raw `Vec<u8>` for signatures and message payloads.

The `browser` feature of `wallet_standard` provides the serde helpers both sides agree on.
