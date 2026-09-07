# The Wallet Standard

The [Wallet Standard](https://github.com/wallet-standard/wallet-standard) is an open protocol that defines how decentralized applications (apps) interact with wallets, independent of any chain or vendor. Instead of every app integrating every wallet SDK, wallets register themselves in the runtime environment and apps discover them.

## Core concepts

### Wallet

A **wallet** is a read-only data object provided to the app. It describes the wallet itself: its `name`, `icon`, the `version` of the standard it implements, the `chains` it supports, the `features` it exposes, and the `accounts` the app is currently authorized to use.

In Rust this maps to the `WalletInfo` trait.

### Account

An **account** is a read-only object the wallet hands to the app, authorizing it to display and act with that account. An account has an `address` (human readable), a `public_key` (raw bytes), its own `chains` and `features`, plus optional `label` and `icon`.

In Rust this maps to the `WalletAccountInfo` trait.

### Feature

A **feature** is a named capability on a wallet, e.g. `standard:connect`, `solana:signMessage` or `experimental:encrypt`. Feature names are namespaced: `standard:*` and `experimental:*` are reserved by the protocol; chains define their own namespaces such as `solana:signTransaction`.

A wallet declares the features it supports and an account declares the subset of features it can be used with.

### Events

Wallets notify the app about changes (accounts changed, chains changed, features changed) through the `standard:events` feature.

## How the protocol maps to Rust

The TypeScript reference implementation describes wallets as plain objects whose `features` map contains named capability objects. Rust has no dynamic object graph, so this crate translates the protocol into **composable traits**:

| Wallet Standard concept              | Rust trait                           |
| ------------------------------------ | ------------------------------------ |
| `Wallet` object metadata             | `WalletInfo`                         |
| `WalletAccount` object               | `WalletAccountInfo`                  |
| `Wallet` + current account           | `Wallet`                             |
| `standard:connect`                   | `WalletStandardConnect`              |
| `standard:disconnect`                | `WalletStandardDisconnect`           |
| `standard:events` (connected wallet) | `ConnectedWalletStandardEvents`      |
| `solana:signMessage`                 | `WalletSolanaSignMessage`            |
| `solana:signTransaction`             | `WalletSolanaSignTransaction`        |
| `solana:signAndSendTransaction`      | `WalletSolanaSignAndSendTransaction` |
| `solana:signIn`                      | `WalletSolanaSignIn`                 |
| `experimental:encrypt`               | `WalletExperimentalEncrypt`          |
| `experimental:decrypt`               | `WalletExperimentalDecrypt`          |

A type that implements `Wallet`, `WalletStandardConnect` and `WalletStandardDisconnect` automatically implements `WalletStandard` — the trait bound apps should target when they want a "full" standard wallet. The `WalletSolana` supertrait bundles the complete Solana feature set, and `wallet_standard::prelude` re-exports everything you need in one import.

## Design principles

1. **Async by default.** Every wallet interaction that could hit user UI (connection prompt, signing confirmation) is `async`.
2. **Errors, not panics.** Wallet failures come back as `WalletError` / `WalletResult<T>`. The only panicking surface is the documented `*_or_panic`-style helpers that unwrap `Option`s for cases the caller has already established.
3. **Output traits, not concrete types.** Signing methods return `impl <OutputTrait>` so wallet implementations can return rich, chain-specific data while apps stay decoupled. `SolanaSignMessageOutput`, `SolanaSignTransactionOutput` etc. are the contracts; your wallet picks the concrete type.
4. **The Solana extension is feature-gated.** Chain-specific code lives behind the `solana` feature so non-Solana wallets only pay for the core protocol.
