# Getting Started

## Installing

Add the crates to your `Cargo.toml`:

<!-- {=install_deps|trim|codeBlock:"toml"} -->

```toml
[dependencies]
# Core protocol traits (required)
wallet_standard = "0.7.1"

# Browser/WASM integration (only for wasm32 targets)
wallet_standard_browser = "0.7.1"
```

<!-- {/install_deps} -->

The core crate is runtime agnostic and compiles everywhere. The `wallet_standard_browser` crate only makes sense inside a WASM module running in a browser.

## Feature flags

### `wallet_standard`

<!-- {=feature_table_core|trim} -->

| Feature                | Description                                                                                                                                                                            |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `solana` _(optional)_  | Enables the Solana feature namespace: signing traits, Solana types, and the `solana:*` chain identifiers. Pulls in the Agave primitives (`solana-keypair`, `solana-transaction`, ...). |
| `browser` _(optional)_ | Enables serde/wasm-bindgen helpers for browser serialization of props and outputs.                                                                                                     |

<!-- {/feature_table_core} -->

### `wallet_standard_browser`

<!-- {=feature_table_browser|trim} -->

| Feature               | Description                                                                     |
| --------------------- | ------------------------------------------------------------------------------- |
| `solana` _(optional)_ | Forwards to `wallet_standard/solana` and adds Solana-specific browser bindings. |

<!-- {/feature_table_browser} -->

## Quick start: an app talking to injected wallets

Inside a WASM application:

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

## Quick start: implementing a wallet

Implement the traits from `wallet_standard` and hand your wallet to the host environment (in the browser via `wallet_standard_browser`, see [Registering a Wallet](./wallets/register.md)):

<!-- {=example_implement_wallet|trim|codeBlock:"rust,ignore"} -->

```rust,ignore
use async_trait::async_trait;
use wallet_standard::prelude::*;

// Define your wallet structure
#[derive(Clone)]
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

// Implement Wallet for your wallet: metadata plus the current account
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
		// Prompt the user and authorize an account, for example:
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

// Implement WalletStandardDisconnect; WalletStandard follows automatically.
#[async_trait(?Send)]
impl WalletStandardDisconnect for MyWallet {
	async fn disconnect(&mut self) -> WalletResult<()> {
		self.current_account = None;
		Ok(())
	}
}
```

<!-- {/example_implement_wallet} -->

## WASM toolchain

When targeting `wasm32-unknown-unknown`, the `getrandom` backend must be configured. This repository ships the required wiring in `.cargo/config.toml`:

```toml
[target.wasm32-unknown-unknown]
rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]
```

and matching target-specific `getrandom` dependencies with the `wasm_js` feature in the crate manifests. Copy both into your project if your app generates keys or randomness on the wasm side.

## Running the tests in this repository

```bash
# native tests for the core crate
devenv shell -c "cargo test_wallet_standard"

# browser tests against a local validator (chrome)
devenv shell -c "test:validator"
```

See [Testing](./testing.md) for the full picture.
