use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, thiserror::Error, Eq, PartialEq, Serialize, Deserialize)]
/// Every failure mode a Wallet Standard interaction can produce.
///
/// The variants map the standard's feature set — connection lifecycle,
/// signing, and the JS boundary — so callers can match on the stage that
/// failed instead of parsing strings.
pub enum WalletError {
	#[error("the arguments provided are not valid")]
	/// The inputs to a feature call were malformed, so the request was
	/// rejected before it could reach the wallet.
	InvalidArguments,
	#[error("icon is not valid")]
	/// The wallet icon failed validation, which would leave apps rendering a
	/// broken or unsafe data URL.
	InvalidIcon,
	#[error("The identifier could not be parsed: {0}")]
	/// A feature or chain identifier did not follow the required
	/// `namespace:method` form; the offending string is carried for
	/// diagnostics.
	InvalidIdentifier(String),
	#[error("The signature is not valid")]
	/// The wallet returned bytes that cannot be an Ed25519 signature, so the
	/// output of the signing feature is unusable.
	InvalidSignature,
	#[error("Signer: {0}")]
	/// A [`solana_signer::Signer`] refused or failed to sign, for example
	/// because its key material is unavailable.
	Signer(String),
	#[error("{0}")]
	/// A JavaScript promise rejected while crossing the wasm boundary; the
	/// message is taken from the rejected `Error` when one is available.
	Js(String),
	#[error("Parsing string failed: {0}")]
	/// Rendering a value as a string failed, which typically means a
	/// display-formatted type could not be produced.
	ParseString(String),
	#[error(transparent)]
	#[cfg(feature = "solana")]
	/// A Solana program rejected an instruction during execution or
	/// preflight simulation.
	Program(#[from] solana_program_error::ProgramError),
	#[error("an error occured during deserialization: {0}")]
	/// A value could not be (de)serialized at the boundary, meaning the
	/// wallet and the app disagreed on the wire shape.
	Serde(String),
	#[cfg(feature = "solana")]
	#[error(transparent)]
	/// The node rejected the transaction itself, independent of signing.
	Transaction(#[from] solana_transaction_error::TransactionError),
	#[error("the requested feature: `{feature}` is not supported for this wallet: `{wallet}`")]
	/// The wallet does not implement a feature the caller requires, naming
	/// both so the app can guide the user to a capable wallet.
	UnsupportedFeature {
		/// The feature the caller requested.
		feature: String,
		/// The wallet that does not provide it.
		wallet: String,
	},
	#[error("icon type is not supported")]
	/// The icon's media type is not one the app can render.
	UnsupportedIconType,
	#[error("The transaction version is not supported by this wallet")]
	/// The wallet only signs certain transaction versions, and this
	/// transaction is not one of them.
	UnsupportedTransactionVersion,
	#[error("Wallet account not connected")]
	/// No account is attached to the wallet yet; connect before requesting
	/// account-scoped operations.
	WalletAccount,
	#[error("The wallet configuration is invalid")]
	/// The wallet configuration is invalid, so the wallet cannot start.
	WalletConfig,
	#[error("An error occurred while connecting to the wallet")]
	/// The connect handshake failed or produced no accounts.
	WalletConnection,
	#[error("Could not decrypt the provided data")]
	/// Decryption failed, for example because the shared key or ciphertext
	/// did not match.
	WalletDecrypt,
	#[error("Action can't be performed because the wallet is disconnected")]
	/// The action requires a connected wallet, but the wallet is
	/// disconnected.
	WalletDisconnected,
	#[error("Error while disconnecting wallet")]
	/// The disconnect handshake failed, leaving the connection state
	/// ambiguous.
	WalletDisconnection,
	#[error("Could not encrypt the provided data")]
	/// Encryption failed before a ciphertext could be produced.
	WalletEncrypt,
	#[error("Wallet keypair")]
	/// The wallet keypair is missing or invalid, so nothing can be signed.
	WalletKeypair,
	#[error("Error loading the wallet")]
	/// The wallet could not be loaded, typically during discovery or
	/// registration.
	WalletLoad,
	#[error("Wallet not connected")]
	/// The wallet reported no connection where one was required.
	WalletNotConnected,
	#[error("The wallet is not yet ready")]
	/// The wallet is still initializing; the caller should retry once it
	/// emits a change event.
	WalletNotReady,
	#[error("Invalid wallet public key")]
	/// The account's public key is absent or not a valid Solana public key.
	WalletPublicKey,
	#[error("Wallet send transaction")]
	/// Broadcasting a signed transaction failed.
	WalletSendTransaction,
	#[error("Wallet sign in")]
	/// A Sign In With Solana attempt failed.
	WalletSignIn,
	#[error("Wallet sign in fields: {0}")]
	/// The Sign In With Solana input fields were invalid; the message names
	/// the problem field.
	WalletSignInFields(String),
	#[error("Wallet sign message")]
	/// Message signing failed or returned no output.
	WalletSignMessage,
	#[error("Wallet sign transaction")]
	/// Transaction signing failed, or the wallet returned wire bytes that
	/// could not be parsed as a transaction.
	WalletSignTransaction,
	#[error("Wallet timeout")]
	/// The wallet or the user did not respond in time, so the request was
	/// abandoned.
	WalletTimeout,
	#[error("Wallet window blocked")]
	/// A popup or prompt was blocked by the browser, usually a permission
	/// or embedded-context restriction.
	WalletWindowBlocked,
	#[error("Wallet window closed")]
	/// The user closed the wallet prompt without approving.
	WalletWindowClosed,
	/// An error from an external source. Implement `IntoWalletError` for your
	/// error to support this functionality.
	#[error("{0}")]
	External(String),
}

impl From<core::fmt::Error> for WalletError {
	fn from(value: core::fmt::Error) -> Self {
		WalletError::ParseString(value.to_string())
	}
}

#[cfg(feature = "browser")]
#[allow(unused_qualifications)]
impl From<wasm_bindgen::JsValue> for WalletError {
	fn from(source: wasm_bindgen::JsValue) -> Self {
		WalletError::Js(js_error_message(&source))
	}
}

/// Render a rejected JavaScript value so failures are never opaque.
///
/// Wallets reject with plain strings, `Error` objects, and `DOMException`
/// values. Only the first survives `JsValue::as_string`, which used to turn
/// every other rejection into the same unhelpful placeholder.
#[cfg(feature = "browser")]
fn js_error_message(value: &wasm_bindgen::JsValue) -> String {
	if let Some(message) = value.as_string() {
		return message;
	}
	if let Ok(message) = js_sys::Reflect::get(value, &wasm_bindgen::JsValue::from_str("message"))
		&& let Some(message) = message.as_string()
	{
		return message;
	}
	format!("An error occurred in the JavaScript: {value:?}")
}
#[cfg(feature = "solana")]
impl From<solana_signer::SignerError> for WalletError {
	fn from(error: solana_signer::SignerError) -> Self {
		WalletError::Signer(error.to_string())
	}
}

#[cfg(feature = "browser")]
impl From<serde_wasm_bindgen::Error> for WalletError {
	fn from(source: serde_wasm_bindgen::Error) -> Self {
		WalletError::Serde(source.to_string())
	}
}

/// The result type used across every wallet interaction so error
/// handling stays uniform for implementors and apps alike.
pub type WalletResult<T> = Result<T, WalletError>;

/// Opt-in bridge for external error types: implementing this empty
/// marker lets `?` convert any displayable error into a
/// [`WalletError::External`] without this crate depending on it.
pub trait IntoWalletError: Display {}

impl<E: IntoWalletError> From<E> for WalletError {
	fn from(value: E) -> Self {
		WalletError::External(value.to_string())
	}
}
