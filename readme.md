# wallet_standard

> The Rust implementation of the [Wallet Standard](https://github.com/wallet-standard/wallet-standard) for [Solana](https://github.com/anza-xyz/wallet-standard).

<p align="center">
  <a href="https://github.com/ifiokjr/wallet_standard/actions?query=workflow:ci">
    <img src="https://github.com/ifiokjr/wallet_standard/actions/workflows/ci.yml/badge.svg" alt="Continuous integration badge for github actions" title="CI Badge" />
  </a>
</p>

<br />

## Description

This repository contains Rust crates that implement the Wallet Standard for Solana, making it easier to interact with Solana in WebAssembly environments:

| Crate                     | Version                                                                            | Description                                                     |
| ------------------------- | ---------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| `wallet_standard`         | <!-- {~core-v:"{{ cargo.workspace.package.version }}"} --><!-- {/core-v} -->       | Core implementation of the wallet standard interface for Solana |
| `wallet_standard_browser` | <!-- {~browser-v:"{{ cargo.workspace.package.version }}"} --><!-- {/browser-v} --> | Browser-specific implementation of the wallet standard          |

### Crate Details

- **wallet_standard**: Provides the core wallet standard interface implementation for Solana. This includes transaction signing, message signing, and other wallet-related functionality.
- **wallet_standard_browser**: Browser-specific implementation of the wallet standard, allowing seamless integration with web applications. Includes JavaScript bindings and browser-specific wallet detection.

## Implementing the Wallet Standard in Your Wallet

This guide is for wallet developers who want to implement the Wallet Standard. There are two main approaches:

1. Build a new wallet with a Wallet Standard compatible API
2. Wrap an existing wallet API with a Wallet Standard compatible API

### For New Wallets

If you're building a new wallet from scratch, implement the traits provided by this library:

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

### For Existing Wallets

If you have an existing wallet API, you can wrap it with the Wallet Standard:

```rust
use wallet_standard_browser::prelude::*;

// Your existing wallet API
struct ExistingWalletAPI {
	// Your existing implementation
}

// Wrapper that implements the Wallet Standard
struct StandardWalletWrapper {
	existing_wallet: ExistingWalletAPI,
	wallet_info: BrowserWalletInfo,
	current_account: Option<BrowserWalletAccountInfo>,
}

impl Wallet for StandardWalletWrapper {
	type Account = BrowserWalletAccountInfo;
	type Wallet = BrowserWalletInfo;

	fn wallet(&self) -> Self::Wallet {
		self.wallet_info.clone()
	}

	fn wallet_account(&self) -> Option<Self::Account> {
		self.current_account.clone()
	}
}

// Implement the required traits by delegating to your existing API
// ...
```

### Registering Your Wallet

In a browser environment, you need to register your wallet so that dApps can discover it:

```rust
use wallet_standard_browser::constants::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn initialize() {
	// Create your wallet implementation
	let my_wallet = MyWallet::new();

	// Register the wallet with the Wallet Standard
	let window = web_sys::window().expect("no global window exists");
	let event = web_sys::CustomEvent::new_with_event_init_dict(
		REGISTER_WALLET_EVENT,
		web_sys::CustomEventInit::new().detail(&JsValue::from(my_wallet)),
	)
	.expect("failed to create custom event");

	window
		.dispatch_event(&event)
		.expect("failed to dispatch event");
}
```

## Using the Wallet Standard in Your dApp

For dApp developers who want to interact with wallets that implement the Wallet Standard:

```rust
use wallet_standard_browser::prelude::*;

async fn connect_wallet() -> WalletResult<()> {
	// Get available wallets
	let wallets = get_wallets().await?;

	// Find a specific wallet by name
	let wallet = wallets
		.get()
		.iter()
		.find(|w| w.name() == "My Wallet")
		.ok_or(WalletError::WalletNotConnected)?;

	// Create a wallet instance
	let mut wallet_instance = BrowserWallet::from(wallet.clone());

	// Connect to the wallet
	let accounts = wallet_instance.connect().await?;
	println!("Connected accounts: {:?}", accounts);

	// Sign a message
	if wallet_instance.connected() {
		let message = b"Hello, Solana!";
		let signature = wallet_instance.sign_message_async(message).await?;
		println!("Signature: {:?}", signature);
	}

	Ok(())
}
```

## Features

Both crates support the following features:

- `solana`: Enables Solana-specific functionality
- `browser`: (for `wallet_standard`) Enables browser-specific functionality

## Examples

Check out the [examples directory](https://github.com/ifiokjr/wallet_standard/tree/main/examples) for complete examples of how to use this library.

## Contributing

[`devenv`](https://devenv.sh/) is used to provide a reproducible development environment for this project. Follow the [getting started instructions](https://devenv.sh/getting-started/).

To automatically load the environment you should [install direnv](https://devenv.sh/automatic-shell-activation/) and then load the `direnv`.

```bash
# The security mechanism didn't allow to load the `.envrc`.
# Since we trust it, let's allow it execution.
direnv allow .
```

At this point you should see the `nix` commands available in your terminal. Any changes made to the `.envrc` file will require you to run the above command again.

Run the following commands to install all the required dependencies.

```bash
install:all
```

This installs all the node dependencies, cargo binaries and solana tooling locally so you don't need to worry about polluting your global namespace.

### Upgrading `devenv`

If you have an outdated version of `devenv` you can update it by running the following commands. If you have an easier way, please create a PR and I'll update these docs.

```bash
nix profile list # find the <index> of the devenv package
nix profile upgrade <index>
```

### Editor Setup

To setup recommended configuration for your favorite editor run the following commands.

```bash
setup:vscode # Setup vscode
```

## License

Unlicense, see the [LICENSE](./license) file.
