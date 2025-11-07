use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, thiserror::Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum WalletError {
	#[error("the arguments provided are not valid")]
	InvalidArguments,
	#[error("icon is not valid")]
	InvalidIcon,
	#[error("The identifier could not be parsed: {0}")]
	InvalidIdentifier(String),
	#[error("The signature is not valid")]
	InvalidSignature,
	#[error("Signer: {0}")]
	Signer(String),
	#[error("{0}")]
	Js(String),
	#[error("Parsing string failed: {0}")]
	ParseString(String),
	#[error(transparent)]
	#[cfg(feature = "solana")]
	Program(#[from] solana_program_error::ProgramError),
	#[error("an error occured during deserialization: {0}")]
	Serde(String),
	#[cfg(feature = "solana")]
	#[error(transparent)]
	Transaction(#[from] solana_transaction::TransactionError),
	#[error("the requested feature: `{feature}` is not supported for this wallet: `{wallet}`")]
	UnsupportedFeature { feature: String, wallet: String },
	#[error("icon type is not supported")]
	UnsupportedIconType,
	#[error("The transaction version is not supported by this wallet")]
	UnsupportedTransactionVersion,
	#[error("Wallet account not connected")]
	WalletAccount,
	#[error("The wallet configuration is invalid")]
	WalletConfig,
	#[error("An error occurred while connecting to the wallet")]
	WalletConnection,
	#[error("Could not decrypt the provided data")]
	WalletDecrypt,
	#[error("Action can't be performed because the wallet is disconnected")]
	WalletDisconnected,
	#[error("Error while disconnecting wallet")]
	WalletDisconnection,
	#[error("Could not encrypt the provided data")]
	WalletEncrypt,
	#[error("Wallet keypair")]
	WalletKeypair,
	#[error("Error loading the wallet")]
	WalletLoad,
	#[error("Wallet not connected")]
	WalletNotConnected,
	#[error("The wallet is not yet ready")]
	WalletNotReady,
	#[error("Invalid wallet public key")]
	WalletPublicKey,
	#[error("Wallet send transaction")]
	WalletSendTransaction,
	#[error("Wallet sign in")]
	WalletSignIn,
	#[error("Wallet sign in fields: {0}")]
	WalletSignInFields(String),
	#[error("Wallet sign message")]
	WalletSignMessage,
	#[error("Wallet sign transaction")]
	WalletSignTransaction,
	#[error("Wallet timeout")]
	WalletTimeout,
	#[error("Wallet window blocked")]
	WalletWindowBlocked,
	#[error("Wallet window closed")]
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
	/// Converts a JavaScript value into a WalletError::Js containing a textual message.
	///
	/// If the provided `JsValue` can be converted to a Rust `String`, that string is used
	/// as the error message; otherwise a default message "An error occurred in the JavaScript."
	/// is used.
	///
	/// # Parameters
	///
	/// - `source`: the JavaScript value to convert into an error message.
	///
	/// # Returns
	///
	/// `WalletError::Js` containing the JS value's string representation, or a default message if none is available.
	///
	/// # Examples
	///
	/// ```
	/// use wasm_bindgen::JsValue;
	/// use crate::error::WalletError;
	///
	/// let js_val = JsValue::from_str("unexpected");
	/// let err = WalletError::from(js_val);
	/// assert_eq!(err, WalletError::Js("unexpected".to_string()));
	/// ```
	#[allow(deprecated)]
	fn from(source: wasm_bindgen::JsValue) -> Self {
		WalletError::Js(
			source
				.as_string()
				.unwrap_or("An error occurred in the JavaScript.".to_string()),
		)
	}
}
#[cfg(feature = "solana")]
impl From<solana_signer::SignerError> for WalletError {
	/// Convert a `solana_signer::SignerError` into the wallet's `Signer` error variant.
	///
	/// The conversion stores the signer's error message as the `String` payload of `WalletError::Signer`.
	///
	/// # Examples
	///
	/// ```
	/// // Demonstrates converting a signer error into `WalletError`.
	/// // `unsafe { std::mem::zeroed() }` is used only to create a value for the example;
	/// // in real code you will have an actual `solana_signer::SignerError`.
	/// # use wallet_standard::error::WalletError;
	/// # // Replace the path below with `solana_signer::SignerError` in real usage.
	/// # let signer_err: solana_signer::SignerError = unsafe { std::mem::zeroed() };
	/// let wallet_err: WalletError = signer_err.into();
	/// match wallet_err {
	///     WalletError::Signer(msg) => assert!(msg.len() >= 0),
	///     _ => panic!("expected WalletError::Signer"),
	/// }
	/// ```
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

pub type WalletResult<T> = Result<T, WalletError>;

pub trait IntoWalletError: Display {}

impl<E: IntoWalletError> From<E> for WalletError {
	fn from(value: E) -> Self {
		WalletError::External(value.to_string())
	}
}