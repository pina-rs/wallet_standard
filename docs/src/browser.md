# Browser Integration

`wallet_standard_browser` runs inside a WASM module in the browser and bridges the JavaScript Wallet Standard both ways:

- **inbound**: JavaScript wallets registered on the page become Rust `BrowserWallet` values.
- **outbound**: Rust wallet implementations become JavaScript wallets registered on the page.

## Discovering wallets

```rust,ignore
use wallet_standard_browser::prelude::*;

let wallets = get_wallets();
let phantom = wallets.get("Phantom").ok_or(WalletError::WalletNotFound)?;

let connected = phantom.connect().await?;
let account = connected.accounts().first().unwrap();
```

`get_wallets()` returns a snapshot of every wallet registered on the page at call time. Each returned `BrowserWallet` implements `Wallet`, `WalletStandardConnect`, `WalletStandardDisconnect` and (with the `solana` feature) the full Solana feature set — including `WalletSolanaSignMessage`, `WalletSolanaSignTransaction`, `WalletSolanaSignAndSendTransaction` and `WalletSolanaSignIn`.

## Registering a Rust wallet into the page

```rust,ignore
use wallet_standard_browser::prelude::*;

let info = BrowserWalletInfo::builder()
    .name("MyWallet".to_string())
    .icon("data:image/svg+xml;base64,...".to_string())
    .chains(vec!["solana:mainnet".to_string()])
    .features(vec![
        "standard:connect".to_string(),
        "solana:signMessage".to_string(),
    ])
    .accounts(vec![])
    .build();

// After this call, every @wallet-standard/app consumer on the
// page (React apps, other extensions) sees your wallet.
register_wallet(&info)?;
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
