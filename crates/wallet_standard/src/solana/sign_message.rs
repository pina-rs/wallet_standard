use async_trait::async_trait;
use futures::future::try_join_all;
use solana_keypair::Keypair;
use solana_signature::Signature;
use solana_signer::Signer;

use crate::WalletResult;

/// Feature identifier for the Solana sign message feature.
///
/// This constant is used to identify the Solana sign message feature in the
/// wallet's feature list. Wallets that implement the `WalletSolanaSignMessage`
/// trait should include this identifier in their feature list.
pub const SOLANA_SIGN_MESSAGE: &str = "solana:signMessage";

/// Trait for outputs that contain a Solana signature.
///
/// This trait provides methods to access a Solana signature, which is typically
/// the result of signing a message or transaction.
///
/// # Example Implementation
///
/// ```rust,ignore
/// struct MySignatureOutput {
///     signature: Signature,
/// }
///
/// impl SolanaSignatureOutput for MySignatureOutput {
///     fn try_signature(&self) -> WalletResult<Signature> {
///         Ok(self.signature)
///     }
///
///     fn signature(&self) -> Signature {
///         self.signature
///     }
/// }
/// ```
pub trait SolanaSignatureOutput {
	/// Returns the signature, or an error if the signature is invalid.
	///
	/// This method allows for error handling when accessing the signature.
	/// If the signature type is provided, the signature must be Ed25519.
	///
	/// # Returns
	///
	/// A `WalletResult` containing the signature if valid,
	/// or a `WalletError` if the signature is invalid.
	fn try_signature(&self) -> WalletResult<Signature>;

	/// Returns the signature.
	///
	/// This method assumes the signature is valid and will panic if it's not.
	/// If the signature type is provided, the signature must be Ed25519.
	///
	/// # Panics
	///
	/// This method will panic if the signature is invalid. Use
	/// `try_signature()` if you want to handle invalid signatures.
	fn signature(&self) -> Signature;
}

/// Implementation of [`SolanaSignatureOutput`] for [`Signature`].
///
/// This allows a Solana Signature to be used directly as a
/// [`SolanaSignatureOutput`].
impl SolanaSignatureOutput for Signature {
	/// Get the contained Solana `Signature`.
	///
	/// This implementation never fails for a plain `Signature` and returns it wrapped in `Ok`.
	///
	/// # Examples
	///
	/// ```
	/// let sig: Signature = unimplemented!(); // obtain a Signature from your context
	/// let result = sig.try_signature();
	/// assert_eq!(result.unwrap(), sig);
	/// ```
	fn try_signature(&self) -> WalletResult<Signature> {
		Ok(*self)
	}

	/// Retrieves the contained Solana signature by value.
	///
	/// # Returns
	///
	/// The contained `Signature`.
	///
	/// # Examples
	///
	/// ```
	/// let sig = Signature::default();
	/// let s = sig.signature();
	/// assert_eq!(s, sig);
	/// ```
	fn signature(&self) -> Signature {
		*self
	}
}

/// Trait for outputs that contain a Solana message signature and the signed
/// message.
///
/// This trait extends [`SolanaSignatureOutput`] to include the message that was
/// signed and optionally the signature type.
///
/// # Example Implementation
///
/// ```rust,ignore
/// struct MySignMessageOutput {
///     signature: Signature,
///     message: Vec<u8>,
/// }
///
/// impl SolanaSignatureOutput for MySignMessageOutput {
///     fn try_signature(&self) -> WalletResult<Signature> {
///         Ok(self.signature)
///     }
///
///     fn signature(&self) -> Signature {
///         self.signature
///     }
/// }
///
/// impl SolanaSignMessageOutput for MySignMessageOutput {
///     fn signed_message(&self) -> Vec<u8> {
///         self.message.clone()
///     }
///
///     fn signature_type(&self) -> Option<String> {
///         Some("ed25519".to_string())
///     }
/// }
/// ```
pub trait SolanaSignMessageOutput: SolanaSignatureOutput {
	/// Returns the message bytes that were signed.
	///
	/// The wallet may prefix or otherwise modify the message before signing it.
	/// This method returns the actual bytes that were signed, which may differ
	/// from the original message.
	fn signed_message(&self) -> Vec<u8>;

	/// Returns the optional type of the message signature produced.
	///
	/// If not provided, the signature must be Ed25519.
	/// This allows for future support of different signature algorithms.
	fn signature_type(&self) -> Option<String>;
}

/// Implementation of [`SolanaSignatureOutput`] for a tuple of (Signature,
/// Vec<u8>, Option<String>).
///
/// This allows a tuple containing a signature, message, and optional signature
/// type to be used as a [`SolanaSignatureOutput`].
impl SolanaSignatureOutput for (Signature, Vec<u8>, Option<String>) {
	/// Extracts the contained Solana `Signature` from the tuple.
	///
	/// # Returns
	///
	/// The contained `Signature` on success, or a `WalletError` if the signature is invalid.
	///
	/// # Examples
	///
	/// ```
	/// let sig: Signature = /* obtain or construct a Signature */ unimplemented!();
	/// let output = (sig, vec![], None::<String>);
	/// let extracted = output.try_signature().unwrap();
	/// assert_eq!(extracted, sig);
	/// ```
	fn try_signature(&self) -> WalletResult<Signature> {
		self.0.try_signature()
	}

	/// Returns the tuple's contained Solana `Signature`, panicking if it is invalid.
	///
	/// # Examples
	///
	/// ```
	/// use solana_sdk::signature::Signature;
	/// // Construct a signature from 64 zero bytes for demonstration purposes.
	/// let sig = Signature::new(&[0u8; 64]);
	/// let tuple = (sig, vec![], None::<String>);
	/// let extracted = tuple.signature();
	/// assert_eq!(extracted, tuple.0);
	/// ```
	fn signature(&self) -> Signature {
		self.0.signature()
	}
}

/// Implementation of [`SolanaSignMessageOutput`] for a tuple of (Signature,
/// Vec<u8>, Option<String>).
///
/// This allows a tuple containing a signature, message, and optional signature
/// type to be used as a [`SolanaSignMessageOutput`].
impl SolanaSignMessageOutput for (Signature, Vec<u8>, Option<String>) {
	/// Returns the bytes that were signed.
	///
	/// A `Vec<u8>` containing the signed message bytes.
	///
	/// # Examples
	///
	/// ```
	/// use wallet_standard::solana::sign_message::SolanaSignMessageOutput;
	/// use solana_sdk::signature::Signature;
	///
	/// let out = (Signature::default(), vec![1, 2, 3], None);
	/// assert_eq!(out.signed_message(), vec![1, 2, 3]);
	/// ```
	fn signed_message(&self) -> Vec<u8> {
		self.1.clone()
	}

	/// Returns the optional signature type associated with the signed message.
	
	///
	
	/// If `None`, the signature should be interpreted as Ed25519.
	
	///
	
	/// # Examples
	
	///
	
	/// ```
	
	/// let out: (solana_sdk::signature::Signature, Vec<u8>, Option<String>) = (solana_sdk::signature::Signature::default(), vec![], Some("ed25519".to_string()));
	
	/// assert_eq!(out.2.clone(), out.signature_type());
	
	/// ```
	fn signature_type(&self) -> Option<String> {
		self.2.clone()
	}
}

/// Trait for wallets that support signing messages with Solana accounts.
///
/// This trait defines methods for signing arbitrary messages using a Solana
/// account's secret key. It provides both single-message and multi-message
/// signing capabilities.
///
/// # Example Implementation
///
/// ```rust,ignore
/// #[async_trait(?Send)]
/// impl WalletSolanaSignMessage for MyWallet {
///     type Output = MySignMessageOutput;
///
///     async fn sign_message_async(&self, message: impl Into<Vec<u8>>) -> WalletResult<Self::Output> {
///         let message_bytes = message.into();
///
///         // In a real implementation, you would use the wallet's signing mechanism
///         let signature = self.sign_with_private_key(&message_bytes)?;
///
///         Ok(MySignMessageOutput {
///             signature,
///             message: message_bytes,
///         })
///     }
///
///     async fn sign_messages<M: Into<Vec<u8>>>(
///         &self,
///         messages: Vec<M>,
///     ) -> WalletResult<Vec<Self::Output>> {
///         let mut results = Vec::new();
///         for message in messages {
///             results.push(self.sign_message_async(message).await?);
///         }
///         Ok(results)
///     }
/// }
/// ```
#[async_trait(?Send)]
pub trait WalletSolanaSignMessage {
	type Output: SolanaSignMessageOutput;

	/// Sign a message using the account's secret key.
	///
	/// This method signs an arbitrary message using the account's secret key.
	/// The message is converted to bytes if it isn't already.
	///
	/// # Parameters
	///
	/// * `message` - The message to sign, which will be converted to bytes
	///
	/// # Returns
	///
	/// A `WalletResult` containing the signature output if successful,
	/// or a `WalletError` if signing fails.
	///
	/// # Errors
	///
	/// This method may return errors such as:
	/// - `WalletError::WalletAccount` if no account is connected
	/// - `WalletError::WalletSignMessage` if signing fails
	/// - `WalletError::InvalidSignature` if the signature is invalid
	async fn sign_message_async(&self, message: impl Into<Vec<u8>>) -> WalletResult<Self::Output>;

	/// Sign multiple messages using the account's secret key.
	///
	/// This method signs multiple arbitrary messages using the account's secret
	/// key. Each message is converted to bytes if it isn't already.
	///
	/// # Parameters
	///
	/// * `messages` - A vector of messages to sign, each of which will be
	///   converted to bytes
	///
	/// # Returns
	///
	/// A `WalletResult` containing a vector of signature outputs if successful,
	/// or a `WalletError` if signing fails.
	///
	/// # Errors
	///
	/// This method may return errors such as:
	/// - `WalletError::WalletAccount` if no account is connected
	/// - `WalletError::WalletSignMessage` if signing fails
	/// - `WalletError::InvalidSignature` if any signature is invalid
	async fn sign_messages<M: Into<Vec<u8>>>(
		&self,
		messages: Vec<M>,
	) -> WalletResult<Vec<Self::Output>>;
}

/// Implementation of WalletSolanaSignMessage for Solana Keypair.
///
/// This allows a Solana Keypair to be used directly as a
/// WalletSolanaSignMessage, which is useful for testing and simple
/// implementations.
#[async_trait(?Send)]
impl WalletSolanaSignMessage for Keypair {
	type Output = (Signature, Vec<u8>, Option<String>);

	async fn sign_message_async(&self, message: impl Into<Vec<u8>>) -> WalletResult<Self::Output> {
		let message: Vec<u8> = message.into();
		let signature = Signer::try_sign_message(self, &message)?;

		Ok((signature, message, None))
	}

	async fn sign_messages<M: Into<Vec<u8>>>(
		&self,
		messages: Vec<M>,
	) -> WalletResult<Vec<Self::Output>> {
		let futures = messages
			.into_iter()
			.map(|message| WalletSolanaSignMessage::sign_message_async(self, message));
		let result = try_join_all(futures).await?;

		Ok(result)
	}
}