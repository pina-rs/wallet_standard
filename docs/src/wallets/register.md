# Registering a Wallet

A wallet implementation starts with three types: an **info** struct implementing `WalletInfo`, an **account** struct implementing `WalletAccountInfo`, and a **wallet** struct implementing `Wallet` that ties them together.

## 1. Wallet metadata

```rust,ignore
use wallet_standard::prelude::*;

#[derive(Clone)]
struct MyWalletInfo {
	accounts: Vec<MyAccount>,
}

impl WalletInfo for MyWalletInfo {
	type Account = MyAccount;

	// Version of the Wallet Standard implemented. The current
	// standard version is "1.0.0".
	fn version(&self) -> String {
		"1.0.0".to_string()
	}

	fn name(&self) -> String {
		"MyWallet".to_string()
	}

	// A data URL is the conventional way to ship icons.
	fn icon(&self) -> String {
		"data:image/svg+xml;base64,...".to_string()
	}

	// CAIP-2 compatible chain identifiers.
	fn chains(&self) -> Vec<String> {
		vec!["solana:mainnet".to_string(), "solana:devnet".to_string()]
	}

	// The features this wallet supports. Only declare what you implement.
	fn features(&self) -> Vec<String> {
		vec![
			"standard:connect".to_string(),
			"standard:disconnect".to_string(),
			"solana:signMessage".to_string(),
			"solana:signTransaction".to_string(),
		]
	}

	// Accounts the app is already authorized to use.
	fn accounts(&self) -> Vec<Self::Account> {
		self.accounts.clone()
	}
}
```

## 2. Account metadata

```rust,ignore
#[derive(Clone)]
struct MyAccount {
	address: String,
	public_key: Vec<u8>,
}

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

	// Per-account feature subset: what this account may be used for.
	fn features(&self) -> Vec<String> {
		vec![
			"solana:signMessage".to_string(),
			"solana:signTransaction".to_string(),
		]
	}

	fn label(&self) -> Option<String> {
		Some("Main Account".to_string())
	}

	fn icon(&self) -> Option<String> {
		None
	}
}
```

## 3. The wallet

```rust,ignore
#[derive(Clone)]
struct MyWallet {
	info: MyWalletInfo,
	account: Option<MyAccount>,
}

impl Wallet for MyWallet {
	type Account = MyAccount;
	type Wallet = MyWalletInfo;

	fn wallet(&self) -> Self::Wallet {
		self.info.clone()
	}

	fn wallet_account(&self) -> Option<Self::Account> {
		self.account.clone()
	}
}
```

## 4. Connect and disconnect

`connect` asks the wallet to authorize accounts for the app; `disconnect` revokes that authorization. Both are async because they may surface user UI.

```rust,ignore
impl WalletStandardConnect for MyWallet {
	async fn connect(&mut self) -> WalletResult<Vec<Self::Account>> {
		// present your authorization UI, then return the
		// accounts the app is authorized to use:
		Ok(vec![/* authorized accounts */])
	}
}

impl WalletStandardDisconnect for MyWallet {
	async fn disconnect(&mut self) -> WalletResult<()> {
		self.account = None;
		Ok(())
	}
}

// WalletStandard is now implemented automatically.
```

Once `Wallet`, `WalletStandardConnect` and `WalletStandardDisconnect` are all implemented, the `WalletStandard` blanket trait applies and your type can be handed to any consumer targeting that bound — including the browser bridge described in [Browser Integration](../browser.md).
