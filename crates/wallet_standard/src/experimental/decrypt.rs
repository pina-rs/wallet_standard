use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

use crate::WalletResult;

/// Feature identifier for the experimental decryption feature.
///
/// The counterpart to [`crate::EXPERIMENTAL_ENCRYPT`]: opens payloads
/// that were encrypted to this wallet's key, keeping the shared secret
/// inside the wallet rather than exposing it to the app.
pub const EXPERIMENTAL_DECRYPT: &str = "experimental:decrypt";

/// The result of an `experimental:decrypt` call.
pub trait ExperimentalDecryptOutput {
	/// `cleartext` that was decrypted.
	fn cleartext(&self) -> Vec<u8>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// The inputs for an `experimental:decrypt` call: the sender's public
/// key, the ciphertext produced by the encrypt feature, and the nonce it
/// was encrypted with.
pub struct ExperimentalDecryptProps {
	/// Cipher to use for decryption.
	#[builder(setter(into))]
	pub cipher: String,
	/// Public key to derive a shared key to decrypt the data using.
	#[builder(setter(into))]
	#[serde(with = "serde_bytes")]
	pub public_key: Vec<u8>,
	/// Ciphertext to decrypt.
	#[builder(setter(into))]
	#[serde(rename = "ciphertext")]
	#[serde(with = "serde_bytes")]
	pub cipher_text: Vec<u8>,
	/// Nonce to use for decryption.
	#[builder(setter(into))]
	#[serde(with = "serde_bytes")]
	pub nonce: Vec<u8>,
	/// Multiple of padding bytes to use for decryption, defaulting to 0.
	///
	/// Valid values `0 | 8 | 16 | 32 | 64 | 128 | 256 | 512 | 1024 | 2048`
	#[builder(default, setter(into, strip_option))]
	pub padding: Option<u8>,
}

#[async_trait(?Send)]
/// Decrypts ciphertexts with the wallet's key material.
pub trait WalletExperimentalDecrypt {
	/// The wallet-specific output carrying the recovered cleartext.
	type Output: ExperimentalDecryptOutput;

	/// Decrypt a batch in one wallet round trip.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::InvalidArguments`] for an empty batch.
	async fn decrypt_many(
		&self,
		props: Vec<ExperimentalDecryptProps>,
	) -> WalletResult<Vec<Self::Output>>;
	/// Decrypt a single ciphertext.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::WalletDecrypt`] when the ciphertext was
	/// not produced for this wallet's key.
	async fn decrypt(&self, props: ExperimentalDecryptProps) -> WalletResult<Self::Output>;
}
