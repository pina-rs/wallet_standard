use async_trait::async_trait;
use wallet_standard::STANDARD_CONNECT;
use wallet_standard::StandardConnectInput;
use wallet_standard::StandardConnectOutput;
use wallet_standard::WalletError;
use wallet_standard::WalletResult;
use wallet_standard::WalletStandardConnect;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::BrowserWallet;
use crate::BrowserWalletAccountInfo;
use crate::impl_feature_from_js;

#[wasm_bindgen]
extern "C" {
	/// apps only ever consume the account list.
	#[derive(Clone, Debug)]
	pub type BrowserStandardConnectOutput;
	/// List of accounts in the `crate::StandardWallet` that the
	/// app has been authorized to use.
	#[wasm_bindgen(method, getter, js_name = accounts)]
	pub fn _accounts(this: &BrowserStandardConnectOutput) -> Vec<BrowserWalletAccountInfo>;
	/// wallet, through which authorization requests are made.
	#[derive(Clone, Debug)]
	pub type StandardConnectFeature;
	/// Version of the connect feature the wallet implements.
	#[wasm_bindgen(method, getter)]
	pub fn version(this: &StandardConnectFeature) -> String;
	/// should go through the typed wrappers that interpret its output.
	#[allow(unused_qualifications)]
	#[wasm_bindgen(method, catch, js_name = connect)]
	pub async fn _connect(
		this: &StandardConnectFeature,
		input: StandardConnectInput,
	) -> Result<JsValue, JsValue>;
}

impl StandardConnectOutput for BrowserStandardConnectOutput {
	type Account = BrowserWalletAccountInfo;

	fn accounts(&self) -> Vec<Self::Account> {
		self._accounts()
	}
}

impl_feature_from_js!(StandardConnectFeature, STANDARD_CONNECT);

impl StandardConnectFeature {
	/// Request authorization with default (interactive) options.
	///
	/// # Errors
	///
	/// Forwards the wallet's rejection; a non-string rejection still
	/// carries its `Error.message`.
	pub async fn connect(&self) -> WalletResult<Vec<BrowserWalletAccountInfo>> {
		self.connect_with_options(StandardConnectInput::default())
			.await
	}

	/// Request authorization, optionally asking only for previously
	/// approved accounts via `silent`, which avoids prompting the user
	/// again on return visits.
	///
	/// # Errors
	///
	/// Forwards the wallet's rejection.
	pub async fn connect_with_options(
		&self,
		options: StandardConnectInput,
	) -> WalletResult<Vec<BrowserWalletAccountInfo>> {
		let result: BrowserStandardConnectOutput = self._connect(options).await?.unchecked_into();

		Ok(result.accounts())
	}
}

#[async_trait(?Send)]
impl WalletStandardConnect for BrowserWallet {
	/// Connect the account and automatically update the attached account.
	async fn connect(&mut self) -> WalletResult<Vec<Self::Account>> {
		self.connect_with_options(StandardConnectInput::default())
			.await
	}

	/// Connect the account and automatically update the attached account.
	#[allow(clippy::manual_async_fn)]
	async fn connect_with_options(
		&mut self,
		options: StandardConnectInput,
	) -> WalletResult<Vec<Self::Account>> {
		let feature = self.wallet.get_feature::<StandardConnectFeature>()?;
		let accounts = feature.connect_with_options(options).await?;
		let account = accounts
			.first()
			.cloned()
			.ok_or(WalletError::WalletConnection)?;
		self.wallet_account = Some(account);

		Ok(accounts)
	}
}
