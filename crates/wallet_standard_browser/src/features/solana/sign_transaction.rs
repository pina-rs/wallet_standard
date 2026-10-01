use async_trait::async_trait;
use js_sys::Array;
use serde::Deserialize;
use serde::Serialize;
use solana_transaction::Transaction;
use solana_transaction::versioned::TransactionVersion;
use solana_transaction::versioned::VersionedTransaction;
use typed_builder::TypedBuilder;

use wallet_standard::SOLANA_SIGN_TRANSACTION;
use wallet_standard::SolanaSignTransactionOptions;
use wallet_standard::SolanaSignTransactionOutput;
use wallet_standard::SolanaSignTransactionProps;
use wallet_standard::WalletError;
use wallet_standard::WalletResult;
use wallet_standard::WalletSolanaSignTransaction;
use wasm_bindgen::JsCast;

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::BrowserWallet;
use crate::BrowserWalletAccountInfo;
use crate::impl_feature_from_js;

#[wasm_bindgen]
extern "C" {
	#[derive(Clone, Debug)]
	/// The JavaScript `{ signedTransaction }` output of a
	/// `solana:signTransaction` call.
	pub type BrowserSolanaSignTransactionOutput;
	/// Signed, serialized transaction, as raw bytes.
	/// Returning a transaction rather than signatures allows multisig wallets,
	/// program wallets, and other wallets that use meta-transactions to return
	/// a modified, signed transaction.
	#[wasm_bindgen(method, getter, js_name = signedTransaction)]
	pub fn _signed_transaction(this: &BrowserSolanaSignTransactionOutput) -> Vec<u8>;
	#[derive(Clone, Debug)]
	/// The JavaScript `solana:signTransaction` feature object of a
	/// registered wallet.
	pub type SolanaSignTransactionFeature;
	/// Version of the feature API.
	#[wasm_bindgen(method, getter)]
	pub fn version(this: &SolanaSignTransactionFeature) -> String;
	/// Raw `supportedTransactionVersions` property read, the JS side of
	/// [`SolanaSignTransactionFeature::supported_transaction_versions`].
	#[wasm_bindgen(method, getter, js_name = supportedTransactionVersions)]
	pub fn _supported_transaction_versions(this: &SolanaSignTransactionFeature) -> Array;
	/// Sign transactions using the account's secret key.
	///
	/// @param inputs Inputs for signing transactions.
	///
	/// @return Outputs of signing transactions.
	#[allow(unused_qualifications)]
	/// The wallet-side JS `signTransaction` method, called with one input
	/// object per transaction.
	#[wasm_bindgen(method, catch, variadic, js_name = signTransaction)]
	pub async fn _sign_transaction(
		this: &SolanaSignTransactionFeature,
		args: Array,
	) -> Result<JsValue, JsValue>;
}

impl SolanaSignTransactionOutput for BrowserSolanaSignTransactionOutput {
	fn signed_transaction_bytes(&self) -> Vec<u8> {
		self._signed_transaction()
	}

	fn signed_transaction(&self) -> WalletResult<VersionedTransaction> {
		let bytes = self.signed_transaction_bytes();

		if let Ok(value) = bincode::deserialize(&bytes) {
			Ok(value)
		} else {
			// A wallet built before versioned transactions signs the unversioned
			// format; accept it so those wallets keep working.
			let transaction: Transaction = bincode::deserialize::<Transaction>(&bytes)
				.map_err(|_| WalletError::WalletSignTransaction)?;

			Ok(transaction.into())
		}
	}
}

impl SolanaSignTransactionFeature {
	/// The transaction versions this wallet will sign.
	///
	/// Checked before any signing call so the app can fail fast instead
	/// of handing the wallet a transaction it must refuse after showing
	/// the user a prompt.
	///
	/// # Errors
	///
	/// Returns [`WalletError::Serde`] when the wallet advertises a version
	/// value this crate does not understand.
	pub fn supported_transaction_versions(&self) -> WalletResult<Vec<TransactionVersion>> {
		let array = self._supported_transaction_versions();

		array
			.iter()
			.map(|value| {
				let version: TransactionVersion = serde_wasm_bindgen::from_value(value)?;
				Ok(version)
			})
			.collect::<WalletResult<Vec<_>>>()
	}
}

impl_feature_from_js!(SolanaSignTransactionFeature, SOLANA_SIGN_TRANSACTION);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// The wire-format input for `solana:signTransaction`: a flat
/// `{ account, transaction, chain, options }` object whose transaction is
/// raw wire bytes, exactly as the JavaScript feature receives it.
///
/// Kept flat on purpose: `serde(flatten)` produces an empty object under
/// `serde-wasm-bindgen`, which once silenced every signing call.
pub struct SolanaSignTransactionInput {
	/// Account to use.
	#[serde(with = "serde_wasm_bindgen::preserve")]
	pub account: BrowserWalletAccountInfo,
	/// Versioned transaction, as serialized wire bytes.
	#[serde(with = "serde_bytes")]
	#[builder(setter(into))]
	pub transaction: Vec<u8>,
	/// Chain to use.
	#[builder(default, setter(into))]
	pub chain: Option<String>,
	/// Additional options for the transaction.
	#[builder(default, setter(into))]
	pub options: Option<SolanaSignTransactionOptions>,
}

impl SolanaSignTransactionFeature {
	/// Sign one transaction with the given account.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::UnsupportedTransactionVersion`] when the
	/// wallet does not sign this transaction's version.
	pub async fn sign_transaction(
		&self,
		account: BrowserWalletAccountInfo,
		props: SolanaSignTransactionProps,
	) -> WalletResult<BrowserSolanaSignTransactionOutput> {
		self.sign_transactions(vec![(account, props)])
			.await?
			.first()
			.cloned()
			.ok_or(WalletError::WalletSignTransaction)
	}

	/// Sign a batch in one wallet round trip, so the user approves a
	/// single prompt for all transactions.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::InvalidArguments`] for an empty batch.
	pub async fn sign_transactions(
		&self,
		inputs: Vec<(BrowserWalletAccountInfo, SolanaSignTransactionProps)>,
	) -> WalletResult<Vec<BrowserSolanaSignTransactionOutput>> {
		if inputs.is_empty() {
			return Err(WalletError::InvalidArguments);
		}

		let supported_transaction_versions = self.supported_transaction_versions()?;
		let inputs = inputs
			.into_iter()
			.map(|(account, props)| {
				// Exit early if any of the versioned transactions are not
				// supported.
				if !supported_transaction_versions.contains(&props.transaction.version()) {
					return Err(WalletError::UnsupportedTransactionVersion);
				}

				let input = SolanaSignTransactionInput::builder()
					.account(account)
					.transaction(
						bincode::serialize(&props.transaction)
							.map_err(|_| WalletError::WalletSignTransaction)?,
					)
					.chain(props.chain)
					.options(props.options)
					.build();

				Ok(input)
			})
			.collect::<WalletResult<Vec<_>>>()?;

		let js_inputs: Array = serde_wasm_bindgen::to_value(&inputs)?.dyn_into()?;
		let js_results: Array = self._sign_transaction(js_inputs).await?.dyn_into()?;

		Ok(js_results.into_iter().map(JsCast::unchecked_into).collect())
	}
}

#[async_trait(?Send)]
impl WalletSolanaSignTransaction for BrowserWallet {
	type Output = BrowserSolanaSignTransactionOutput;

	async fn sign_transaction(
		&self,
		props: SolanaSignTransactionProps,
	) -> WalletResult<Self::Output> {
		let Some(ref wallet_account) = self.wallet_account else {
			return Err(WalletError::WalletAccount);
		};

		self.wallet
			.get_feature::<SolanaSignTransactionFeature>()?
			.sign_transaction(wallet_account.clone(), props)
			.await
	}

	async fn sign_transactions(
		&self,
		inputs: Vec<SolanaSignTransactionProps>,
	) -> WalletResult<Vec<Self::Output>> {
		let Some(ref wallet_account) = self.wallet_account else {
			return Err(WalletError::WalletAccount);
		};

		let inputs = inputs
			.into_iter()
			.map(|props| (wallet_account.clone(), props))
			.collect();

		self.wallet
			.get_feature::<SolanaSignTransactionFeature>()?
			.sign_transactions(inputs)
			.await
	}
}
