# Getting Started

## Installing

Add the crates to your `Cargo.toml`:

```toml
[dependencies]
# Core protocol traits (required)
wallet_standard = "0.5"

# Browser/WASM integration (only for wasm32 targets)
wallet_standard_browser = "0.5"
```

The core crate is runtime agnostic and compiles everywhere. The `wallet_standard_browser` crate only makes sense inside a WASM module running in a browser.

## Feature flags

### `wallet_standard`

| Feature                | Description                                                                                                                                                                            |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `solana` _(optional)_  | Enables the Solana feature namespace: signing traits, Solana types, and the `solana:*` chain identifiers. Pulls in the Agave primitives (`solana-keypair`, `solana-transaction`, ...). |
| `browser` _(optional)_ | Enables serde/wasm-bindgen helpers for browser serialization of props and outputs.                                                                                                     |

```toml
wallet_standard = { version = "0.5", features = ["solana"] }
```

### `wallet_standard_browser`

| Feature               | Description                                                                     |
| --------------------- | ------------------------------------------------------------------------------- |
| `solana` _(optional)_ | Forwards to `wallet_standard/solana` and adds Solana-specific browser bindings. |

## Quick start: an app talking to injected wallets

Inside a WASM application:

```rust,ignore
use wallet_standard_browser::prelude::*;

// Discover every Wallet Standard wallet registered in the page.
let wallets = get_wallets();

for (name, wallet) in wallets.iter() {
    log::info!("found wallet {name}: {}", wallet.icon());
}

// Pick one and connect.
let mut wallet = wallets.get("Phantom").expect("phantom is installed");
let output = wallet.connect().await?;

// The authorized account.
let account = output.accounts().first().unwrap();
log::info!("connected as {}", account.address());
```

## Quick start: implementing a wallet

Implement the traits from `wallet_standard` and hand your wallet to the host environment (in the browser via `wallet_standard_browser`, see [Registering a Wallet](./wallets/register.md)):

```rust,ignore
use wallet_standard::prelude::*;

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

impl WalletStandardConnect for MyWallet {
	type Output = StandardConnectOutputProperties;

	async fn connect(&mut self) -> WalletResult<Self::Output> {
		// show your UI, authorize the account ...
	}
}

// WalletStandard is implemented automatically once you also
// implement WalletStandardDisconnect.
```

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
