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
cargo add wallet_standard_browser
```

Or directly add the following to your `Cargo.toml`:

```toml
[dependencies]
wallet_standard_browser = "0.5.0"
```

### Features

| Feature  | Description                           |
| -------- | ------------------------------------- |
| `solana` | Enables Solana-specific functionality |

## Core Components

This crate provides several key components:

1. **`BrowserWallet`**: A wrapper around JavaScript wallet implementations that conforms to the Wallet Standard
2. **`BrowserWalletInfo`**: Represents wallet metadata for browser-based wallets
3. **`BrowserWalletAccountInfo`**: Represents account information for browser-based wallets
4. **Feature wrappers**: JavaScript bindings for wallet features like connect, disconnect, sign message, etc.

## Usage for dApp Developers

### Detecting and Connecting to Wallets

```rust
use wallet_standard_browser::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

// Function to detect and connect to available wallets
async fn connect_to_wallet() -> WalletResult<()> {
	// Get available wallets
	let wallets = get_wallets().await?;

	// Log the number of available wallets
	console::log_1(&format!("Found {} wallets", wallets.get().len()).into());

	// Find a specific wallet by name
	if let Some(wallet) = wallets.get().iter().find(|w| w.name() == "Phantom") {
		console::log_1(&format!("Found Phantom wallet").into());

		// Create a wallet instance
		let mut wallet_instance = BrowserWallet::from(wallet.clone());

		// Connect to the wallet
		let accounts = wallet_instance.connect().await?;
		console::log_1(&format!("Connected to {} accounts", accounts.len()).into());

		// Now you can use the wallet for operations
		if wallet_instance.connected() {
			// Example: Sign a message
			let message = b"Hello, Solana!";
			let signature = wallet_instance.sign_message_async(message).await?;
			console::log_1(&format!("Message signed successfully").into());
		}
	} else {
		console::log_1(&"Phantom wallet not found".into());
	}

	Ok(())
}

// Call this function from your application
fn initialize() {
	spawn_local(async {
		if let Err(err) = connect_to_wallet().await {
			console::error_1(&format!("Error: {:?}", err).into());
		}
	});
}
```

### Listening for Wallet Events

```rust
use wallet_standard_browser::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

// Create a closure to handle wallet registration events
fn listen_for_wallets() -> WalletResult<()> {
	let wallets_callback = Closure::wrap(Box::new(move |wallet: BrowserWalletInfo| {
		// A new wallet has been registered
		let wallet_name = wallet.name();
		web_sys::console::log_1(&format!("New wallet registered: {}", wallet_name).into());

		// You can now use this wallet
		let wallet_instance = BrowserWallet::from(wallet);
		// Store the wallet instance for later use
	}) as Box<dyn FnMut(BrowserWalletInfo)>);

	// Get the wallets registry
	spawn_local(async move {
		match get_wallets().await {
			Ok(wallets) => {
				// Register the callback for new wallet registrations
				let _dispose = wallets.on_register(&wallets_callback);

				// Keep the callback alive
				wallets_callback.forget();
			}
			Err(err) => {
				web_sys::console::error_1(&format!("Error getting wallets: {:?}", err).into());
			}
		}
	});

	Ok(())
}
```

### Signing and Sending Transactions (Solana)

```rust
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use solana_transaction::VersionedTransaction;
use wallet_standard_browser::prelude::*;
use wasm_bindgen_futures::spawn_local;

async fn send_transaction(wallet: &mut BrowserWallet) -> WalletResult<()> {
	// Ensure the wallet is connected
	if !wallet.connected() {
		wallet.connect().await?;
	}

	// Get the wallet's public key
	let pubkey = wallet.try_solana_pubkey()?;

	// Create a simple transaction (transfer 0.001 SOL to self)
	let instructions = vec![solana_system_interface::instructions::transfer(
		&pubkey, &pubkey, 1_000_000, // lamports (0.001 SOL)
	)];

	// Create a message
	let message = Message::new(&instructions, Some(&pubkey));

	// Create a transaction
	let transaction = Transaction::new_unsigned(message);

	// Convert to versioned transaction
	let versioned_transaction = VersionedTransaction::from(transaction);

	// Create transaction props
	let props = SolanaSignAndSendTransactionProps::builder()
		.transaction(versioned_transaction)
		.build();

	// Sign and send the transaction
	let result = wallet.sign_and_send_transaction(props).await?;

	// Get the signature
	let signature = result.signature();
	web_sys::console::log_1(&format!("Transaction sent with signature: {}", signature).into());

	Ok(())
}
```

## Usage for Wallet Developers

### Implementing a Browser Wallet

If you're developing a wallet that needs to be compatible with the Wallet Standard in a browser environment, you'll need to:

1. Create a JavaScript wallet implementation that conforms to the Wallet Standard
2. Register your wallet with the Wallet Standard

Here's a simplified example of how to register your wallet:

```rust
use wallet_standard_browser::constants::*;
use wasm_bindgen::prelude::*;
use web_sys::CustomEvent;
use web_sys::CustomEventInit;
use web_sys::window;

#[wasm_bindgen]
pub fn register_wallet() {
	// Create your wallet implementation
	let wallet_info = create_wallet_info();

	// Register the wallet with the Wallet Standard
	let window = window().expect("no global window exists");
	let event_init = CustomEventInit::new();
	event_init.detail(&JsValue::from_serde(&wallet_info).unwrap());

	let event = CustomEvent::new_with_event_init_dict(REGISTER_WALLET_EVENT, &event_init)
		.expect("failed to create custom event");

	window
		.dispatch_event(&event)
		.expect("failed to dispatch event");
}

fn create_wallet_info() -> serde_json::Value {
	// Create a wallet info object that conforms to the Wallet Standard
	serde_json::json!({
		"name": "My Wallet",
		"icon": "data:image/svg+xml;base64,...", // Base64 encoded SVG
		"version": "1.0.0",
		"chains": ["solana:mainnet"],
		"features": [
			"standard:connect",
			"standard:disconnect",
			"solana:signMessage",
			"solana:signTransaction",
			"solana:signAndSendTransaction"
		],
		"accounts": []
	})
}
```

### Implementing Wallet Features

For each feature your wallet supports, you'll need to implement the corresponding JavaScript functions. Here's an example for the `solana:signMessage` feature:

```javascript
// In your wallet's JavaScript code
window.myWallet = {
    // ... other wallet properties
    features: {
        // ... other features
        "solana:signMessage": {
            version: "1.0.0",
            signMessage: async function(accounts, messages) {
                // Implement message signing
                return messages.map(message => ({
                    signature: new Uint8Array(...), // The signature bytes
                    signedMessage: message, // The message that was signed
                    signatureType: "ed25519" // Optional signature type
                }));
            }
        }
    }
};
```

Then in your Rust code, you can use the `wallet_standard_browser` crate to interact with this JavaScript implementation:

```rust
use wallet_standard_browser::prelude::*;
use wasm_bindgen_futures::spawn_local;

fn use_wallet() {
	spawn_local(async {
		// Get available wallets
		let wallets = get_wallets().await.unwrap();

		// Find your wallet
		if let Some(wallet) = wallets.get().iter().find(|w| w.name() == "My Wallet") {
			// Create a wallet instance
			let mut wallet_instance = BrowserWallet::from(wallet.clone());

			// Connect to the wallet
			wallet_instance.connect().await.unwrap();

			// Sign a message
			let message = b"Hello, Solana!";
			let signature = wallet_instance.sign_message_async(message).await.unwrap();

			// Use the signature
			web_sys::console::log_1(&format!("Signature: {:?}", signature.signature()).into());
		}
	});
}
```

## Examples

For complete examples of how to use this crate, check out the [examples directory](https://github.com/ifiokjr/wallet_standard/tree/main/examples) in the repository.

## API Reference

For detailed API documentation, please refer to the [API documentation](https://docs.rs/wallet_standard_browser/).

[crate-image]: https://img.shields.io/crates/v/wallet_standard_browser.svg
[crate-link]: https://crates.io/crates/wallet_standard_browser
[docs-image]: https://docs.rs/wallet_standard_browser/badge.svg
[docs-link]: https://docs.rs/wallet_standard_browser/
[ci-status-image]: https://github.com/ifiokjr/wallet_standard/workflows/ci/badge.svg
[ci-status-link]: https://github.com/ifiokjr/wallet_standard/actions?query=workflow:ci
[unlicense-image]: https://img.shields.io/badge/license-Unlicence-blue.svg
[unlicense-link]: https://opensource.org/license/unlicense
[codecov-image]: https://codecov.io/github/ifiokjr/wallet_standard/graph/badge.svg?token=87K799Q78I
[codecov-link]: https://codecov.io/github/ifiokjr/wallet_standard
