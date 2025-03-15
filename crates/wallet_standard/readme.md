# `wallet_standard`

<br />

> An implementation of the Solana wallet standard in Rust.

<br />

[![Crate][crate-image]][crate-link] [![Docs][docs-image]][docs-link] [![Status][ci-status-image]][ci-status-link] [![Unlicense][unlicense-image]][unlicense-link] [![codecov][codecov-image]][codecov-link]

## Overview

The `wallet_standard` crate provides a Rust implementation of the [Wallet Standard](https://github.com/wallet-standard/wallet-standard) for Solana. It defines a set of traits and types that create a consistent interface for wallets and dApps to interact with the Solana blockchain.

## Installation

To install you can use the following command:

```bash
cargo add wallet_standard
```

Or directly add the following to your `Cargo.toml`:

```toml
[dependencies]
wallet_standard = "0.4" # replace with the latest version
```

### Features

| Feature   | Description                                                      |
| --------- | ---------------------------------------------------------------- |
| `browser` | Enables browser-specific functionality with wasm-bindgen support |
| `solana`  | Enables Solana-specific functionality                            |

## Core Concepts

The Wallet Standard defines several key concepts:

1. **Wallet**: A wallet is a software application that manages accounts and can perform operations like signing transactions.
2. **Account**: An account represents a user's identity on the blockchain, typically associated with a public/private key pair.
3. **Features**: Capabilities that a wallet provides, such as connecting, signing messages, or signing transactions.

## Key Traits

### Core Traits

- `Wallet`: The base trait for all wallet implementations
- `WalletInfo`: Provides information about the wallet (name, icon, supported chains)
- `WalletAccountInfo`: Provides information about wallet accounts
- `WalletStandard`: Combines the core wallet functionality

### Standard Features

- `WalletStandardConnect`: For connecting to a wallet and authorizing accounts
- `WalletStandardDisconnect`: For disconnecting from a wallet
- `ConnectedWalletStandardEvents`: For listening to wallet events

### Solana-Specific Traits

- `WalletSolanaSignMessage`: For signing arbitrary messages
- `WalletSolanaSignTransaction`: For signing transactions
- `WalletSolanaSignAndSendTransaction`: For signing and sending transactions
- `WalletSolanaSignIn`: For implementing Sign-In With Solana (SIWS)

### Experimental Features

- `WalletExperimentalEncrypt`: For encrypting data
- `WalletExperimentalDecrypt`: For decrypting data

## Usage Examples

### Implementing a Basic Wallet

```rust
use async_trait::async_trait;
use wallet_standard::prelude::*;

// Define your wallet structure
struct MyWallet {
	name: String,
	icon: String,
	accounts: Vec<MyAccount>,
	current_account: Option<MyAccount>,
}

// Define your account structure
#[derive(Clone)]
struct MyAccount {
	address: String,
	public_key: Vec<u8>,
}

// Implement WalletAccountInfo for your account
impl WalletAccountInfo for MyAccount {
	fn address(&self) -> String {
		self.address.clone()
	}

	fn public_key(&self) -> Vec<u8> {
		self.public_key.clone()
	}

	fn chains(&self) -> Vec<String> {
		vec!["solana:mainnet".to_string()]
	}

	fn features(&self) -> Vec<String> {
		vec![
			STANDARD_CONNECT.to_string(),
			STANDARD_DISCONNECT.to_string(),
			SOLANA_SIGN_MESSAGE.to_string(),
		]
	}

	fn label(&self) -> Option<String> {
		Some("My Account".to_string())
	}

	fn icon(&self) -> Option<String> {
		None
	}
}

// Implement WalletInfo for your wallet
impl WalletInfo for MyWallet {
	type Account = MyAccount;

	fn version(&self) -> String {
		"1.0.0".to_string()
	}

	fn name(&self) -> String {
		self.name.clone()
	}

	fn icon(&self) -> String {
		self.icon.clone()
	}

	fn chains(&self) -> Vec<String> {
		vec!["solana:mainnet".to_string()]
	}

	fn features(&self) -> Vec<String> {
		vec![
			STANDARD_CONNECT.to_string(),
			STANDARD_DISCONNECT.to_string(),
			SOLANA_SIGN_MESSAGE.to_string(),
		]
	}

	fn accounts(&self) -> Vec<Self::Account> {
		self.accounts.clone()
	}
}

// Implement Wallet for your wallet
impl Wallet for MyWallet {
	type Account = MyAccount;
	type Wallet = Self;

	fn wallet(&self) -> Self::Wallet {
		self.clone()
	}

	fn wallet_account(&self) -> Option<Self::Account> {
		self.current_account.clone()
	}
}

// Implement WalletStandardConnect
#[async_trait(?Send)]
impl WalletStandardConnect for MyWallet {
	async fn connect(&mut self) -> WalletResult<Vec<Self::Account>> {
		// Implement connection logic
		// For example, prompt the user to select an account
		if let Some(account) = self.accounts.first().cloned() {
			self.current_account = Some(account.clone());
			Ok(vec![account])
		} else {
			Err(WalletError::WalletConnection)
		}
	}

	async fn connect_with_options(
		&mut self,
		_options: StandardConnectInput,
	) -> WalletResult<Vec<Self::Account>> {
		self.connect().await
	}
}

// Implement WalletStandardDisconnect
#[async_trait(?Send)]
impl WalletStandardDisconnect for MyWallet {
	async fn disconnect(&mut self) -> WalletResult<()> {
		self.current_account = None;
		Ok(())
	}
}
```

### Implementing Solana-Specific Features

```rust
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signature;
use solana_sdk::signer::Signer;
use wallet_standard::prelude::*;

// Assuming MyWallet is defined as above

// Define a custom output type for sign message
struct MySignMessageOutput {
	signature: Signature,
	signed_message: Vec<u8>,
}

impl SolanaSignatureOutput for MySignMessageOutput {
	fn try_signature(&self) -> WalletResult<Signature> {
		Ok(self.signature)
	}

	fn signature(&self) -> Signature {
		self.signature
	}
}

impl SolanaSignMessageOutput for MySignMessageOutput {
	fn signed_message(&self) -> Vec<u8> {
		self.signed_message.clone()
	}

	fn signature_type(&self) -> Option<String> {
		None
	}
}

// Implement WalletSolanaSignMessage
#[async_trait(?Send)]
impl WalletSolanaSignMessage for MyWallet {
	type Output = MySignMessageOutput;

	async fn sign_message_async(&self, message: impl Into<Vec<u8>>) -> WalletResult<Self::Output> {
		let message_bytes = message.into();

		// In a real implementation, you would use the wallet's signing mechanism
		// This is just a placeholder example using a keypair
		let keypair = Keypair::new(); // In reality, this would be the user's keypair
		let signature = keypair.sign_message(&message_bytes);

		Ok(MySignMessageOutput {
			signature,
			signed_message: message_bytes,
		})
	}

	async fn sign_messages<M: Into<Vec<u8>>>(
		&self,
		messages: Vec<M>,
	) -> WalletResult<Vec<Self::Output>> {
		let mut results = Vec::new();
		for message in messages {
			results.push(self.sign_message_async(message).await?);
		}
		Ok(results)
	}
}
```

## Error Handling

The library provides a comprehensive error handling system through the `WalletError` enum:

```rust
use wallet_standard::prelude::*;

fn handle_wallet_operation() -> WalletResult<()> {
	// Attempt some wallet operation
	let result = some_wallet_function();

	match result {
		Ok(_) => Ok(()),
		Err(e) => {
			match e {
				WalletError::WalletNotConnected => {
					// Handle not connected error
					Err(WalletError::WalletNotConnected)
				}
				WalletError::InvalidSignature => {
					// Handle invalid signature error
					Err(WalletError::InvalidSignature)
				}
				// Handle other error types
				_ => Err(e),
			}
		}
	}
}
```

## Advanced Usage

For more advanced usage, including implementing experimental features or custom wallet behaviors, please refer to the [API documentation](https://docs.rs/wallet_standard/).

A full example of how to use this crate can be found in the [wallet_standard_browser](https://github.com/ifiokjr/wallet_standard/tree/main/crates/wallet_standard_browser) crate.

[crate-image]: https://img.shields.io/crates/v/wallet_standard.svg
[crate-link]: https://crates.io/crates/wallet_standard
[docs-image]: https://docs.rs/wallet_standard/badge.svg
[docs-link]: https://docs.rs/wallet_standard/
[ci-status-image]: https://github.com/ifiokjr/wallet_standard/workflows/ci/badge.svg
[ci-status-link]: https://github.com/ifiokjr/wallet_standard/actions?query=workflow:ci
[unlicense-image]: https://img.shields.io/badge/license-Unlicence-blue.svg
[unlicense-link]: https://opensource.org/license/unlicense
[codecov-image]: https://codecov.io/github/ifiokjr/wallet_standard/graph/badge.svg?token=87K799Q78I
[codecov-link]: https://codecov.io/github/ifiokjr/wallet_standard
