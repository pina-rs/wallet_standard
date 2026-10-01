use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use serde_bytes::ByteBuf;
use typed_builder::TypedBuilder;
use wallet_standard::SOLANA_SIGN_OFFCHAIN_MESSAGE;
use wallet_standard::SolanaOffchainMessageVersion;
use wallet_standard::SolanaSignOffchainMessageOutput;
use wallet_standard::SolanaSignatureOutput;
use wallet_standard::WalletError;
use wallet_standard::WalletResult;
use wallet_standard::WalletSolanaSignOffchainMessage;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::BrowserWallet;
use crate::BrowserWalletAccountInfo;
use crate::impl_feature_from_js;

#[wasm_bindgen]
extern "C" {
	/// The JavaScript `{ signedOffchainMessage, signature }` output of a
	/// `solana:signOffchainMessage` call.
	#[derive(Clone, Debug)]
	pub type BrowserSolanaSignOffchainMessageOutput;
	/// Full preamble and body bytes the wallet constructed and signed.
	#[wasm_bindgen(method, getter, js_name = signedOffchainMessage)]
	pub fn _signed_offchain_message(this: &BrowserSolanaSignOffchainMessageOutput) -> Vec<u8>;
	/// Message signature produced.
	#[wasm_bindgen(method, getter)]
	pub fn _signature(this: &BrowserSolanaSignOffchainMessageOutput) -> Vec<u8>;
	/// Optional signature algorithm marker; absent means Ed25519.
	#[wasm_bindgen(method, getter, js_name = signatureType)]
	pub fn _signature_type(this: &BrowserSolanaSignOffchainMessageOutput) -> Option<String>;
	/// The JavaScript `solana:signOffchainMessage` feature object of a
	/// registered wallet, through which SRFC-3 offchain messages are
	/// constructed and signed.
	#[derive(Clone, Debug)]
	pub type SolanaSignOffchainMessageFeature;
	/// Version of the feature API.
	#[wasm_bindgen(method, getter)]
	pub fn version(this: &SolanaSignOffchainMessageFeature) -> String;
	/// Raw `supportedMessageVersions` property read, the JS side of
	/// [`SolanaSignOffchainMessageFeature::supported_message_versions`].
	#[wasm_bindgen(method, getter, js_name = supportedMessageVersions)]
	pub fn supported_message_versions_getter(
		this: &SolanaSignOffchainMessageFeature,
	) -> js_sys::Array;
	/// The wallet-side JS `signOffchainMessage` method, called with one
	/// input object per message.
	#[allow(unused_qualifications)]
	#[wasm_bindgen(method, catch, variadic, js_name = signOffchainMessage)]
	pub async fn _sign_offchain_message(
		this: &SolanaSignOffchainMessageFeature,
		args: js_sys::Array,
	) -> Result<JsValue, JsValue>;
}

impl SolanaSignatureOutput for BrowserSolanaSignOffchainMessageOutput {
	fn try_signature(&self) -> WalletResult<solana_signature::Signature> {
		self._signature()
			.try_into()
			.map_err(|_| WalletError::InvalidSignature)
	}

	fn signature(&self) -> solana_signature::Signature {
		self.try_signature().unwrap_throw()
	}
}

impl SolanaSignOffchainMessageOutput for BrowserSolanaSignOffchainMessageOutput {
	fn signed_offchain_message(&self) -> Vec<u8> {
		self._signed_offchain_message()
	}

	fn signature_type(&self) -> Option<String> {
		self._signature_type()
	}
}

impl_feature_from_js!(
	SolanaSignOffchainMessageFeature,
	SOLANA_SIGN_OFFCHAIN_MESSAGE
);

impl SolanaSignOffchainMessageFeature {
	/// The offchain message versions this wallet can construct and sign,
	/// checked before any call so unsupported versions fail before the
	/// user sees a prompt.
	///
	/// # Errors
	///
	/// Returns [`WalletError::Serde`] for a version value this crate does
	/// not understand.
	pub fn supported_message_versions(&self) -> WalletResult<Vec<SolanaOffchainMessageVersion>> {
		self.supported_message_versions_getter()
			.iter()
			.map(|value| {
				let version: SolanaOffchainMessageVersion = serde_wasm_bindgen::from_value(value)?;
				Ok(version)
			})
			.collect()
	}

	/// Sign one offchain message with the given account.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::UnsupportedMessageVersion`] when the
	/// wallet does not sign this message's version and forwards the
	/// wallet's rejection otherwise.
	pub async fn sign_offchain_message(
		&self,
		account: BrowserWalletAccountInfo,
		props: wallet_standard::SolanaSignOffchainMessageProps,
	) -> WalletResult<BrowserSolanaSignOffchainMessageOutput> {
		self.sign_offchain_messages(vec![(account, props)])
			.await?
			.first()
			.cloned()
			.ok_or(WalletError::WalletSignMessage)
	}

	/// Sign a batch in one wallet round trip, so the user approves one
	/// prompt for every message.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::InvalidArguments`] for an empty batch.
	pub async fn sign_offchain_messages(
		&self,
		inputs: Vec<(
			BrowserWalletAccountInfo,
			wallet_standard::SolanaSignOffchainMessageProps,
		)>,
	) -> WalletResult<Vec<BrowserSolanaSignOffchainMessageOutput>> {
		if inputs.is_empty() {
			return Err(WalletError::InvalidArguments);
		}

		let supported_message_versions = self.supported_message_versions()?;
		let inputs = inputs
			.into_iter()
			.map(|(account, props)| {
				if !supported_message_versions.contains(&props.message_version) {
					return Err(WalletError::UnsupportedMessageVersion);
				}

				Ok(SolanaSignOffchainMessageInput::builder()
					.account(account)
					.message_version(props.message_version)
					.message(props.message)
					.required_signers(
						props
							.required_signers
							.into_iter()
							.map(ByteBuf::from)
							.collect::<Vec<_>>(),
					)
					.build())
			})
			.collect::<WalletResult<Vec<_>>>()?;

		let js_inputs: js_sys::Array = serde_wasm_bindgen::to_value(&inputs)?.dyn_into()?;
		let js_results: js_sys::Array = self._sign_offchain_message(js_inputs).await?.dyn_into()?;

		Ok(js_results
			.into_iter()
			.map(wasm_bindgen::JsCast::unchecked_into)
			.collect())
	}
}

/// The wire-format input for `solana:signOffchainMessage`: a flat
/// `{ messageVersion, account, message, requiredSigners }` object whose
/// signer keys are `Uint8Array`s, exactly as the JavaScript feature
/// receives it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct SolanaSignOffchainMessageInput {
	/// The SRFC-3 message format version to construct.
	pub message_version: SolanaOffchainMessageVersion,
	/// Account to sign with; its public key must appear in the required
	/// signers.
	#[serde(with = "serde_wasm_bindgen::preserve")]
	pub account: BrowserWalletAccountInfo,
	/// UTF-8 message body the wallet wraps in the canonical preamble.
	#[builder(setter(into))]
	pub message: String,
	/// Public keys (32 raw bytes each) that must sign.
	pub required_signers: Vec<ByteBuf>,
}

#[async_trait(?Send)]
impl WalletSolanaSignOffchainMessage for BrowserWallet {
	type Output = BrowserSolanaSignOffchainMessageOutput;

	/// Sign one offchain message with the connected account.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::WalletAccount`] when no account is
	/// attached and forwards the wallet's rejection otherwise.
	async fn sign_offchain_message(
		&self,
		props: wallet_standard::SolanaSignOffchainMessageProps,
	) -> WalletResult<Self::Output> {
		self.sign_offchain_messages(vec![props])
			.await?
			.first()
			.cloned()
			.ok_or(WalletError::WalletSignMessage)
	}

	/// Sign a batch in one wallet round trip.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::WalletAccount`] when no account is
	/// attached and [`WalletError::InvalidArguments`] for an empty batch.
	async fn sign_offchain_messages(
		&self,
		props: Vec<wallet_standard::SolanaSignOffchainMessageProps>,
	) -> WalletResult<Vec<Self::Output>> {
		let Some(ref wallet_account) = self.wallet_account else {
			return Err(WalletError::WalletAccount);
		};

		let inputs = props
			.into_iter()
			.map(|props| (wallet_account.clone(), props))
			.collect();

		self.wallet
			.get_feature::<SolanaSignOffchainMessageFeature>()?
			.sign_offchain_messages(inputs)
			.await
	}
}
