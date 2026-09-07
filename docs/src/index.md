# Wallet Standard

`wallet_standard` is a [Wallet Standard](./wallet-standard.md) implementation in Rust with first-class WASM support. It lets you:

- **Write wallets** in Rust that behave like native Wallet Standard participants, using traits that mirror the reference TypeScript definitions one-to-one.
- **Consume browser wallets** from Rust/WASM applications through `wallet_standard_browser`, which bridges the JavaScript Wallet Standard registered in the page into typed Rust traits.
- **Target any runtime** — the core `wallet_standard` crate is platform agnostic; only the `browser` crate touches the DOM.

The Wallet Standard is a chain-agnostic protocol for app↔wallet interaction: apps discover wallets that inject themselves into the environment, and wallets expose **features** (like `standard:connect` or `solana:signTransaction`) that apps can call without knowing the wallet's internals.

## What you get in this book

- What the Wallet Standard is and how the protocol maps to Rust traits.
- Setup instructions for apps and wallet implementors.
- A tour of the crates and their feature gates.
- Recipes for registering a wallet, signing messages and transactions, and Sign in With Solana.
- Browser integration details, including the JavaScript bridge.
- Testing, security, and the development workflow used in this repository.

## Where to go next

- New to the protocol? Start with [The Wallet Standard](./wallet-standard.md).
- Building an app that talks to injected wallets? See [Browser Integration](./browser.md).
- Implementing a wallet in Rust? See [Registering a Wallet](./wallets/register.md).

## Crates

| Crate                                                                         | Description                                                                |
| ----------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| [`wallet_standard`](https://crates.io/crates/wallet_standard)                 | Chain-agnostic traits and types implementing the Wallet Standard protocol. |
| [`wallet_standard_browser`](https://crates.io/crates/wallet_standard_browser) | WASM bindings that bridge the JavaScript Wallet Standard into Rust.        |
