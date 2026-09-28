# Documentation providers

Content here is the single source of truth. Consumers in the readmes, the
mdBook pages, and rustdoc comments reference these blocks with consumer
tags named after the provider; `mdt update` syncs them and `mdt check`
fails CI when a copy drifts.

<!-- {@install_deps} -->

[dependencies]
# Core protocol traits (required)
wallet_standard = "{{ cargo.workspace.package.version }}"

# Browser/WASM integration (only for wasm32 targets)
wallet_standard_browser = "{{ cargo.workspace.package.version }}"

<!-- {/install_deps} -->

<!-- {@feature_table_core} -->

| Feature                | Description                                                                                                                                                                            |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `solana` _(optional)_  | Enables the Solana feature namespace: signing traits, Solana types, and the `solana:*` chain identifiers. Pulls in the Agave primitives (`solana-keypair`, `solana-transaction`, ...). |
| `browser` _(optional)_ | Enables serde/wasm-bindgen helpers for browser serialization of props and outputs.                                                                                                     |

<!-- {/feature_table_core} -->

<!-- {@feature_table_browser} -->

| Feature               | Description                                                                     |
| --------------------- | ------------------------------------------------------------------------------- |
| `solana` _(optional)_ | Forwards to `wallet_standard/solana` and adds Solana-specific browser bindings. |

<!-- {/feature_table_browser} -->

<!-- {@trait_mapping_table} -->

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

<!-- {/trait_mapping_table} -->

<!-- {@example_implement_wallet} -->

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

<!-- {/example_implement_wallet} -->

<!-- {@example_detect_connect} -->

use wallet_standard_browser::prelude::*;
use wasm_bindgen_futures::spawn_local;

fn detect_and_connect() {
	spawn_local(async {
		let wallets = get_wallets();

		// Find a wallet by name and connect to it.
		if let Some(info) = wallets.get().iter().find(|wallet| wallet.name() == "Phantom") {
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

<!-- {/example_detect_connect} -->

<!-- {@example_register_wallet} -->

use wallet_standard_browser::prelude::*;

// Build a JS wallet object whose `features` map holds feature objects with
// Rust-backed JS callbacks (see the example crate for the full construction).
let wallet = build_wallet_object(/* … */);

// Every @wallet-standard/app consumer on the page — React apps, other
// extensions — now sees the wallet.
register_wallet(&wallet)?;

<!-- {/example_register_wallet} -->

<!-- {@sign_and_send_rationale} -->

`solana:signAndSendTransaction` is the safer counterpart to
`solana:signTransaction`: the wallet signs **and broadcasts**, so a
malicious app never holds a signed transaction it could replay or
redirect. Apps that need to aggregate signatures, batch, or relay
transactions themselves use `solana:signTransaction` instead — wallets
that refuse to hand back signed payloads simply do not implement it.

<!-- {/sign_and_send_rationale} -->
