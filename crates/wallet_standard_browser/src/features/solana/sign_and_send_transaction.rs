use async_trait::async_trait;
use js_sys::Array;
use serde::Deserialize;
use serde::Serialize;
use solana_signature::Signature;
use solana_transaction::versioned::TransactionVersion;
use solana_transaction::versioned::VersionedTransaction;
use typed_builder::TypedBuilder;
use wallet_standard::SOLANA_SIGN_AND_SEND_TRANSACTION;
use wallet_standard::SolanaSignAndSendTransactionOptions;
use wallet_standard::SolanaSignAndSendTransactionProps;
use wallet_standard::SolanaSignatureOutput;
use wallet_standard::WalletError;
use wallet_standard::WalletResult;
use wallet_standard::WalletSolanaSignAndSendTransaction;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::BrowserWallet;
use crate::BrowserWalletAccountInfo;
use crate::impl_feature_from_js;

#[wasm_bindgen]
extern "C" {
	#[derive(Clone, Debug)]
	/// The JavaScript `{ signature }` output of a
	/// `solana:signAndSendTransaction` call; the signature is the only
	/// thing an app receives when the wallet owns broadcasting.
	pub type BrowserSolanaSignAndSendTransactionOutput;
	/// Transaction signature, as raw bytes.
	#[wasm_bindgen(method, getter, js_name = signature)]
	pub fn _signature(this: &BrowserSolanaSignAndSendTransactionOutput) -> Vec<u8>;
	#[derive(Clone, Debug)]
	/// The JavaScript `solana:signAndSendTransaction` feature object of a
	/// registered wallet.
	pub type SolanaSignAndSendTransactionFeature;
	/// Version of the feature API.
	#[wasm_bindgen(method, getter)]
	pub fn version(this: &SolanaSignAndSendTransactionFeature) -> String;
	#[wasm_bindgen(method, getter, js_name = supportedTransactionVersions)]
	/// Raw `supportedTransactionVersions` property read, the JS side of
	/// [`SolanaSignAndSendTransactionFeature::supported_transaction_versions`].
	pub fn supported_transaction_versions_getter(
		this: &SolanaSignAndSendTransactionFeature,
	) -> Array;
	/// Sign transactions using the account's secret key and send them to the
	/// chain.
	///
	/// @param inputs Inputs for signing and sending transactions.
	///
	/// @return Outputs of signing and sending transactions.
	#[allow(unused_qualifications)]
	/// The wallet-side JS `signAndSendTransaction` method, called with one
	/// input object per transaction.
	#[wasm_bindgen(method, catch, variadic, js_name = signAndSendTransaction)]
	pub async fn _sign_and_send_transaction(
		this: &SolanaSignAndSendTransactionFeature,
		args: Array,
	) -> Result<JsValue, JsValue>;
}

impl SolanaSignatureOutput for BrowserSolanaSignAndSendTransactionOutput {
	fn try_signature(&self) -> WalletResult<Signature> {
		self._signature()
			.try_into()
			.map_err(|_| WalletError::InvalidSignature)
	}

	fn signature(&self) -> Signature {
		self.try_signature().unwrap_throw()
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// The wire-format input for `solana:signAndSendTransaction`: a flat
/// `{ account, transaction, chain, options }` object whose transaction is
/// raw wire bytes, exactly as the JavaScript feature receives it.
pub struct SolanaSignAndSendTransactionInput {
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
	/// Options for signing and sending.
	#[builder(default, setter(into))]
	pub options: Option<SolanaSignAndSendTransactionOptions>,
}

impl SolanaSignAndSendTransactionFeature {
	/// The transaction versions this wallet will sign and send.
	///
	/// Checked before any call so unsupported versions fail before the
	/// user sees a prompt.
	///
	/// # Errors
	///
	/// Returns [`WalletError::Serde`] for a version value this crate does
	/// not understand.
	pub fn supported_transaction_versions(&self) -> WalletResult<Vec<TransactionVersion>> {
		let array = self.supported_transaction_versions_getter();

		array
			.iter()
			.map(|value| {
				let version: TransactionVersion = serde_wasm_bindgen::from_value(value)?;
				Ok(version)
			})
			.collect::<WalletResult<Vec<_>>>()
	}

	/// Sign and send one transaction; the wallet broadcasts it itself.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::UnsupportedTransactionVersion`] when the
	/// wallet does not sign this transaction's version.
	pub async fn sign_and_send_transaction(
		&self,
		account: BrowserWalletAccountInfo,
		props: SolanaSignAndSendTransactionProps,
	) -> WalletResult<BrowserSolanaSignAndSendTransactionOutput> {
		let input = SolanaSignAndSendTransactionInput::builder()
			.account(account)
			.transaction(
				bincode::serialize(&props.transaction)
					.map_err(|_| WalletError::WalletSignTransaction)?,
			)
			.chain(props.chain)
			.options(props.options)
			.build();

		self.sign_and_send_transactions(vec![input])
			.await?
			.first()
			.cloned()
			.ok_or(WalletError::WalletSignTransaction)
	}

	/// Sign and send a batch in one wallet round trip.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::InvalidArguments`] for an empty batch
	/// and [`WalletError::UnsupportedTransactionVersion`] when one of the
	/// transactions uses a version the wallet will not sign.
	pub async fn sign_and_send_transactions(
		&self,
		inputs: Vec<SolanaSignAndSendTransactionInput>,
	) -> WalletResult<Vec<BrowserSolanaSignAndSendTransactionOutput>> {
		if inputs.is_empty() {
			return Err(WalletError::InvalidArguments);
		}

		let supported_transaction_versions = self.supported_transaction_versions()?;

		for input in &inputs {
			// Exit early if any of the versioned transactions are not
			// supported.
			let transaction: VersionedTransaction = bincode::deserialize(&input.transaction)
				.map_err(|_| WalletError::WalletSignTransaction)?;
			if !supported_transaction_versions.contains(&transaction.version()) {
				return Err(WalletError::UnsupportedTransactionVersion);
			}
		}

		let js_inputs: Array = serde_wasm_bindgen::to_value(&inputs)?.dyn_into()?;
		let js_results: Array = self
			._sign_and_send_transaction(js_inputs)
			.await?
			.dyn_into()?;

		Ok(js_results
			.into_iter()
			.map(wasm_bindgen::JsCast::unchecked_into)
			.collect())
	}
}

impl_feature_from_js!(
	SolanaSignAndSendTransactionFeature,
	SOLANA_SIGN_AND_SEND_TRANSACTION
);

#[async_trait(?Send)]
impl WalletSolanaSignAndSendTransaction for BrowserWallet {
	type Output = BrowserSolanaSignAndSendTransactionOutput;

	async fn sign_and_send_transaction(
		&self,
		props: SolanaSignAndSendTransactionProps,
	) -> WalletResult<Self::Output> {
		let Some(ref wallet_account) = self.wallet_account else {
			return Err(WalletError::WalletAccount);
		};

		self.wallet
			.get_feature::<SolanaSignAndSendTransactionFeature>()?
			.sign_and_send_transaction(wallet_account.clone(), props)
			.await
	}

	async fn sign_and_send_transactions(
		&self,
		inputs: Vec<SolanaSignAndSendTransactionProps>,
	) -> WalletResult<Vec<Self::Output>> {
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
			.get_feature::<SolanaSignAndSendTransactionFeature>()?
			.sign_and_send_transactions(inputs)
			.await
	}
}
