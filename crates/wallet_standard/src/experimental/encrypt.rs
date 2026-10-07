use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

use crate::WalletResult;

/// Feature identifier for the experimental encryption feature.
///
/// Exists so two Wallet Standard parties can establish a shared secret
/// through the wallet instead of a separate key-exchange step; tagged
/// experimental because the cipher set and field names are still
/// settling across implementations.
pub const EXPERIMENTAL_ENCRYPT: &str = "experimental:encrypt";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// The inputs for an `experimental:encrypt` call.
///
/// Encryption is performed against a counterparty's public key with an
/// ephemeral nonce, yielding ciphertext only the counterparty's wallet
/// can open.
pub struct ExperimentalEncryptProps {
	/// Cipher to use for encryption.
	#[builder(setter(into))]
	pub cipher: String,
	/// Public key to derive a shared key to encrypt the data using.
	#[builder(setter(into))]
	#[serde(with = "serde_bytes")]
	pub public_key: Vec<u8>,
	/// Cleartext to encrypt.
	#[serde(with = "serde_bytes")]
	pub cleartext: Vec<u8>,
	/// Multiple of padding bytes to use for encryption, defaulting to 0.
	///
	/// Valid values `0 | 8 | 16 | 32 | 64 | 128 | 256 | 512 | 1024 | 2048`
	#[builder(default, setter(into, strip_option))]
	pub padding: Option<u8>,
}

/// The result of an `experimental:encrypt` call.
///
/// Both the ciphertext and the nonce are returned because the recipient
/// needs the pair to decrypt; the nonce is safe to transport alongside
/// the ciphertext.
pub trait ExperimentalEncryptOutput {
	/// Ciphertext that was encrypted.
	fn cipher_text(&self) -> Vec<u8>;
	/// Nonce that was used for encryption.
	fn nonce(&self) -> Vec<u8>;
}

#[async_trait(?Send)]
/// Encrypts cleartexts with the wallet's key material.
pub trait WalletExperimentalEncrypt {
	/// The wallet-specific output carrying the ciphertext and nonce.
	type Output: ExperimentalEncryptOutput;

	/// Encrypt a batch in one wallet round trip.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::InvalidArguments`] for an empty batch.
	async fn encrypt_many(
		&self,
		props: Vec<ExperimentalEncryptProps>,
	) -> WalletResult<Vec<Self::Output>>;
	/// Encrypt a single cleartext.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::WalletEncrypt`] when the wallet cannot
	/// derive a shared key for the given public key.
	async fn encrypt(&self, props: ExperimentalEncryptProps) -> WalletResult<Self::Output>;
}
