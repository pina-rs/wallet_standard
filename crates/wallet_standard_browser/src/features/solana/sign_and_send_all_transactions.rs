use async_trait::async_trait;
use js_sys::Array;
use js_sys::Reflect;
use wallet_standard::SOLANA_SIGN_AND_SEND_ALL_TRANSACTIONS;
use wallet_standard::SolanaSignAndSendAllTransactionsOptions;
use wallet_standard::SolanaSignAndSendTransactionProps;
use wallet_standard::WalletError;
use wallet_standard::WalletResult;
use wallet_standard::WalletSettled;
use wallet_standard::WalletSolanaSignAndSendAllTransactions;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::BrowserWallet;
use crate::SolanaSignAndSendTransactionInput;
use crate::impl_feature_from_js;

#[wasm_bindgen]
extern "C" {
	/// The JavaScript `solana:signAndSendAllTransactions` feature object
	/// of a registered wallet, through which whole batches are signed,
	/// sent, and settled per input.
	#[derive(Clone, Debug)]
	pub type SolanaSignAndSendAllTransactionsFeature;
	/// Version of the feature API.
	#[wasm_bindgen(method, getter)]
	pub fn version(this: &SolanaSignAndSendAllTransactionsFeature) -> String;
	/// Raw `supportedTransactionVersions` property read, the JS side of
	/// [`SolanaSignAndSendAllTransactionsFeature::supported_transaction_versions`].
	#[wasm_bindgen(method, getter, js_name = supportedTransactionVersions)]
	pub fn supported_transaction_versions_getter(
		this: &SolanaSignAndSendAllTransactionsFeature,
	) -> Array;
	/// The wallet-side JS `signAndSendAllTransactions` method. Unlike the
	/// single-shot features it takes the input array and the batch
	/// options as two arguments and settles every input independently.
	#[allow(unused_qualifications)]
	#[wasm_bindgen(method, catch, js_name = signAndSendAllTransactions)]
	pub async fn _sign_and_send_all_transactions(
		this: &SolanaSignAndSendAllTransactionsFeature,
		inputs: Array,
		options: &JsValue,
	) -> Result<JsValue, JsValue>;
}

impl SolanaSignAndSendAllTransactionsFeature {
	/// The transaction versions this wallet will sign and send, checked
	/// before any call so unsupported versions fail before the user sees
	/// a prompt.
	///
	/// # Errors
	///
	/// Returns [`WalletError::Serde`] for a version value this crate does
	/// not understand.
	pub fn supported_transaction_versions(
		&self,
	) -> WalletResult<Vec<solana_transaction::versioned::TransactionVersion>> {
		self.supported_transaction_versions_getter()
			.iter()
			.map(|value| {
				let version: solana_transaction::versioned::TransactionVersion =
					serde_wasm_bindgen::from_value(value)?;
				Ok(version)
			})
			.collect()
	}

	/// Sign and send every input in one wallet round trip, settling each
	/// input independently: a rejected transaction arrives as a
	/// [`WalletSettled::Rejected`] element instead of failing the batch.
	///
	/// # Errors
	///
	/// Fails only when the wallet rejects the request outright.
	pub async fn sign_and_send_all_transactions(
		&self,
		inputs: Vec<SolanaSignAndSendTransactionInput>,
		options: SolanaSignAndSendAllTransactionsOptions,
	) -> WalletResult<Vec<WalletSettled<BrowserSolanaSignAndSendAllTransactionsOutput>>> {
		if inputs.is_empty() {
			return Err(WalletError::InvalidArguments);
		}

		let supported_transaction_versions = self.supported_transaction_versions()?;
		for input in &inputs {
			let transaction: solana_transaction::versioned::VersionedTransaction =
				bincode::deserialize(&input.transaction)
					.map_err(|_| WalletError::WalletSignTransaction)?;
			if !supported_transaction_versions.contains(&transaction.version()) {
				return Err(WalletError::UnsupportedTransactionVersion);
			}
		}

		let js_inputs: Array = serde_wasm_bindgen::to_value(&inputs)?.dyn_into()?;
		let js_options = serde_wasm_bindgen::to_value(&options)?;
		let js_results: Array = self
			._sign_and_send_all_transactions(js_inputs, &js_options)
			.await?
			.dyn_into()?;

		js_results
			.iter()
			.map(|element| {
				let status = Reflect::get(&element, &JsValue::from_str("status"))
					.map_err(WalletError::from)?;
				match status.as_string().as_deref() {
					Some("fulfilled") => {
						let value = Reflect::get(&element, &JsValue::from_str("value"))
							.map_err(WalletError::from)?;
						Ok(WalletSettled::Fulfilled {
							value: value.unchecked_into(),
						})
					}
					Some("rejected") => {
						let reason = Reflect::get(&element, &JsValue::from_str("reason"))
							.map_err(WalletError::from)?;
						Ok(WalletSettled::Rejected {
							reason: WalletError::from(reason),
						})
					}
					other => {
						Err(WalletError::Serde(format!(
							"unexpected settlement status `{other:?}`"
						)))
					}
				}
			})
			.collect()
	}
}

impl_feature_from_js!(
	SolanaSignAndSendAllTransactionsFeature,
	SOLANA_SIGN_AND_SEND_ALL_TRANSACTIONS
);

/// The per-transaction JavaScript output of a
/// `solana:signAndSendAllTransactions` call, carrying the broadcast
/// signature.
pub type BrowserSolanaSignAndSendAllTransactionsOutput =
	crate::BrowserSolanaSignAndSendTransactionOutput;

#[async_trait(?Send)]
impl WalletSolanaSignAndSendAllTransactions for BrowserWallet {
	type Output = BrowserSolanaSignAndSendAllTransactionsOutput;

	/// Sign and send a batch with the connected account, settling every
	/// transaction independently through the JS wallet.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::WalletAccount`] when no account is
	/// attached; per-transaction failures arrive as settled elements.
	async fn sign_and_send_all_transactions(
		&self,
		inputs: Vec<SolanaSignAndSendTransactionProps>,
		options: SolanaSignAndSendAllTransactionsOptions,
	) -> WalletResult<Vec<WalletSettled<Self::Output>>> {
		let Some(ref wallet_account) = self.wallet_account else {
			return Err(WalletError::WalletAccount);
		};

		let inputs = inputs
			.into_iter()
			.map(|props| {
				Ok(SolanaSignAndSendTransactionInput::builder()
					.account(wallet_account.clone())
					.transaction(
						bincode::serialize(&props.transaction)
							.map_err(|_| WalletError::WalletSignTransaction)?,
					)
					.chain(props.chain)
					.options(props.options)
					.build())
			})
			.collect::<WalletResult<Vec<_>>>()?;

		self.wallet
			.get_feature::<SolanaSignAndSendAllTransactionsFeature>()?
			.sign_and_send_all_transactions(inputs, options)
			.await
	}
}
