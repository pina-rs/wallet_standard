# `wallet_standard_browser`

<br />

> The WebAssembly/browser compatible Rust implementation of the Wallet Standard.

<br />

[![Crate][crate-image]][crate-link] [![Docs][docs-image]][docs-link] [![Status][ci-status-image]][ci-status-link] [![Unlicense][unlicense-image]][unlicense-link] [![codecov][codecov-image]][codecov-link]

## Overview

The `wallet_standard_browser` crate provides a WebAssembly-compatible implementation of the [Wallet Standard](https://github.com/wallet-standard/wallet-standard) for Solana. It enables Rust-based wallets to be used in browser environments and allows dApps to interact with these wallets through a consistent interface.

## Installation

To install you can use the following command:

```bash
cargo add wallet_standard_browser --features solana
```

Or directly add the following to your `Cargo.toml`:

<!-- {=install_deps|trim|codeBlock:"toml"} -->

```toml
[dependencies]
# Core protocol traits (required)
wallet_standard = "0.7.1"

# Browser/WASM integration (only for wasm32 targets)
wallet_standard_browser = "0.7.1"
```

<!-- {/install_deps} -->

Enable the Solana feature namespace when you depend on the browser crate:

### Features

<!-- {=feature_table_browser|trim} -->

| Feature               | Description                                                                     |
| --------------------- | ------------------------------------------------------------------------------- |
| `solana` _(optional)_ | Forwards to `wallet_standard/solana` and adds Solana-specific browser bindings. |

<!-- {/feature_table_browser} -->

## Core Components

This crate provides several key components:

1. **`BrowserWallet`**: A wrapper around JavaScript wallet implementations that conforms to the Wallet Standard
2. **`BrowserWalletInfo`**: Represents wallet metadata for browser-based wallets
3. **`BrowserWalletAccountInfo`**: Represents account information for browser-based wallets
4. **Feature wrappers**: JavaScript bindings for wallet features like connect, disconnect, sign message, etc.

## Usage for dApp Developers

### Detecting and Connecting to Wallets

`get_wallets()` is synchronous and returns a snapshot of every wallet registered on the page. Each wallet is wrapped in a `BrowserWallet`, which implements the same traits as the core crate — `WalletStandardConnect`, `WalletStandardDisconnect`, and (with the `solana` feature) the full Solana feature set.

<!-- {=example_detect_connect|trim|codeBlock:"rust,ignore"} -->

```rust,ignore
use wallet_standard_browser::prelude::*;
use wasm_bindgen_futures::spawn_local;

fn detect_and_connect() {
	spawn_local(async {
		let wallets = get_wallets();

		// Find a wallet by name and connect to it.
		if let Some(info) = wallets
			.get()
			.iter()
			.find(|wallet| wallet.name() == "Phantom")
		{
			let mut wallet = BrowserWallet::from(info.clone());
			let accounts = wallet.connect().await?;

			let account = accounts
				.first()
				.cloned()
				.ok_or(WalletError::WalletConnection)?;
			web_sys::console::log_1(&format!("Connected to {}", account.address()).into());
		}

		Ok::<(), WalletError>(())
	});
}
```

<!-- {/example_detect_connect} -->

### Listening for Wallet Events

```rust,ignore
use wallet_standard_browser::prelude::*;
use wasm_bindgen::prelude::*;

fn listen_for_wallets() {
	let wallets = get_wallets();

	let callback = Closure::new(move |wallet: BrowserWalletInfo| {
		web_sys::console::log_1(&format!("New wallet registered: {}", wallet.name()).into());
		// Wrap the wallet and store it for later use.
		let _wallet = BrowserWallet::from(wallet);
	});

	// Returns the unsubscribe handle; keep the closure alive for as long as
	// you want to receive registrations.
	let _dispose = wallets.on_register(&callback);
}
```

### Signing and Sending Transactions (Solana)

```rust,ignore
use solana_message::Message;
use solana_message::VersionedMessage;
use solana_pubkey::Pubkey;
use solana_transaction::versioned::VersionedTransaction;
use wallet_standard::SolanaSignatureOutput;
use wallet_standard_browser::prelude::*;

async fn send_transaction(wallet: &mut BrowserWallet) -> WalletResult<()> {
	// Ensure the wallet is connected
	if !wallet.connected() {
		wallet.connect().await?;
	}

	let pubkey = wallet.try_solana_pubkey()?;

	let instruction = solana_system_interface::instruction::transfer(
		&pubkey, &pubkey, 1_000_000, // lamports
	);
	let message = VersionedMessage::Legacy(Message::new(&[instruction], Some(&pubkey)));
	let transaction = VersionedTransaction {
		signatures: vec![solana_signature::Signature::default()],
		message,
	};

	let props = SolanaSignAndSendTransactionProps::builder()
		.transaction(transaction)
		.chain("solana:mainnet")
		.build();

	let result = wallet.sign_and_send_transaction(props).await?;
	let signature = result.try_signature()?;
	web_sys::console::log_1(&format!("Transaction sent: {signature}").into());

	Ok(())
}
```

## Usage for Wallet Developers

A Wallet Standard wallet implemented in Rust must be exposed to the page as a JavaScript object — including feature objects whose methods are real JS functions backed by Rust closures — and registered with [`register_wallet`](https://docs.rs/wallet-standard-wallet). The example crate [`examples/crates/surfpool-wallet-core`](https://github.com/pina-rs/wallet_standard/tree/main/examples) builds such a wallet end to end (connect, disconnect, events, signing, and sending) and the Leptos and Dioxus examples exercise it against a local [surfpool](https://github.com/solana-foundation/surfpool) node under Playwright.

<!-- {=example_register_wallet|trim|codeBlock:"rust,ignore"} -->

```rust,ignore
use wallet_standard_browser::prelude::*;

// Build a JS wallet object whose `features` map holds feature objects with
// Rust-backed JS callbacks (see the example crate for the full construction).
let wallet = build_wallet_object(/* … */);

// Every @wallet-standard/app consumer on the page — React apps, other
// extensions — now sees the wallet.
register_wallet(&wallet)?;
```

<!-- {/example_register_wallet} -->

## Examples

For complete examples of how to use this crate, check out the [examples directory](https://github.com/pina-rs/wallet_standard/tree/main/examples) in the repository: a Leptos dApp, a Dioxus dApp, and a Playwright suite that runs both against a surfpool backend.

## API Reference

For detailed API documentation, please refer to the [API documentation](https://docs.rs/wallet_standard_browser/).

[crate-image]: https://img.shields.io/crates/v/wallet_standard_browser.svg
[crate-link]: https://crates.io/crates/wallet_standard_browser
[docs-image]: https://docs.rs/wallet_standard_browser/badge.svg
[docs-link]: https://docs.rs/wallet_standard_browser/
[ci-status-image]: https://github.com/pina-rs/wallet_standard/workflows/ci/badge.svg
[ci-status-link]: https://github.com/pina-rs/wallet_standard/actions?query=workflow:ci
[unlicense-image]: https://img.shields.io/badge/license-Unlicence-blue.svg
[unlicense-link]: https://opensource.org/license/unlicense
[codecov-image]: https://codecov.io/github/pina-rs/wallet_standard/graph/badge.svg?token=87K799Q78I
[codecov-link]: https://codecov.io/github/pina-rs/wallet_standard
