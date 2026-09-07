# Crates and Features

## Repository layout

```
wallet_standard/
├── crates/
│   ├── wallet_standard/           # core protocol traits, chain agnostic
│   └── wallet_standard_browser/   # wasm bindings for the browser
├── docs/                          # this book
└── setup/                         # local test tooling (webdriver, keypairs, ...)
```

## `wallet_standard`

The protocol crate. Everything here is runtime agnostic: the same traits compile on native platforms (desktop wallet implementations, servers) and on `wasm32`.

### Modules

- `standard` — the `standard:*` namespace: `connect`, `disconnect`, `events`.
- `solana` _(feature `solana`)_ — the `solana:*` namespace: `sign_message`, `sign_transaction`, `sign_and_send_transaction`, `sign_in`.
- `experimental` — the `experimental:*` namespace: `encrypt`, `decrypt` (x25519-xsalsa20-poly1305).
- `error` — `WalletError` and `WalletResult<T>`.

### Key traits

| Trait                                                     | Purpose                                                                       |
| --------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `WalletInfo`                                              | Wallet metadata: `version`, `name`, `icon`, `chains`, `features`, `accounts`. |
| `WalletAccountInfo`                                       | Account data: `address`, `public_key`, `chains`, `features`, `label`, `icon`. |
| `Wallet`                                                  | Links a wallet to its currently connected account.                            |
| `WalletStandard`                                          | Blanket trait over `Wallet` + connect + disconnect.                           |
| `WalletStandardConnect`                                   | `standard:connect` — request account authorization.                           |
| `WalletStandardDisconnect`                                | `standard:disconnect` — revoke authorization.                                 |
| `ConnectedWalletStandardEvents`                           | `standard:events` — subscribe to wallet/account changes.                      |
| `WalletSolana` _(solana)_                                 | Blanket trait over all Solana features below.                                 |
| `WalletSolanaSignMessage` _(solana)_                      | Sign arbitrary byte messages.                                                 |
| `WalletSolanaSignTransaction` _(solana)_                  | Sign transactions without broadcasting.                                       |
| `WalletSolanaSignAndSendTransaction` _(solana)_           | Sign and broadcast in one step.                                               |
| `WalletSolanaSignIn` _(solana)_                           | Sign in With Solana (SIWS).                                                   |
| `WalletExperimentalEncrypt` / `WalletExperimentalDecrypt` | End-to-end encryption of messages to an account.                              |

### Output traits

Signing methods do not return concrete structs. They return types implementing **output traits**:

- `SolanaSignatureOutput` — the signed message and signature.
- `SolanaSignMessageOutput` — adds signed-message metadata (`signature_type`, `signed_message`, ...).
- `SolanaSignTransactionOutput` — the signed transaction bytes.
- `SolanaSignAndSendTransactionOutput` — a `SolanaSignatureOutput` (the signature confirms the send).
- `SolanaSignInOutput` — a `SolanaSignMessageOutput` with the SIWS input echoed back.

This keeps apps decoupled from wallet internals while letting wallets expose richer data through their own concrete output types.

## `wallet_standard_browser`

The WASM crate. It performs three jobs:

1. **Bridging** — binds the JavaScript `@wallet-standard/app` and `@wallet-standard/wallet` reference implementations (bundled under `js/`) into Rust via `wasm-bindgen`.
2. **Discovering wallets** — `get_wallets()` returns every wallet registered in the page.
3. **Publishing wallets** — `register_wallet()` makes a Rust-implemented wallet visible to every JavaScript Wallet Standard consumer on the page.

Everything the browser crate exposes implements the same traits from `wallet_standard`, so application code written against the trait objects works identically for browser wallets and your own Rust wallets.

The browser crate depends on `wallet_standard` with the `browser` feature enabled, which activates the serde/wasm-bindgen serialization helpers used to move props and outputs across the JS boundary.
