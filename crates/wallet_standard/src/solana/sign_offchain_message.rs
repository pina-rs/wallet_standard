use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

use crate::SolanaSignatureOutput;
use crate::WalletResult;

/// Feature identifier for the Solana sign-offchain-message feature.
///
/// Implements the Solana offchain message signing specification
/// (SRFC-3): the wallet wraps the app's text in a canonical preamble —
/// domain, signer set, version — before signing, so a signed "message"
/// can never be replayed as a transaction, and verifiers need no wallet
/// SDK to check it.
pub const SOLANA_SIGN_OFFCHAIN_MESSAGE: &str = "solana:signOffchainMessage";

/// The offchain message specification version this input targets.
///
/// Version 1 is the only version defined by SRFC-3; the wallet rejects
/// anything it does not advertise in `supportedMessageVersions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolanaOffchainMessageVersion {
	/// The initial SRFC-3 message format.
	V1,
}

impl From<SolanaOffchainMessageVersion> for u8 {
	fn from(version: SolanaOffchainMessageVersion) -> Self {
		match version {
			SolanaOffchainMessageVersion::V1 => 1,
		}
	}
}

impl Serialize for SolanaOffchainMessageVersion {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_u8(u8::from(*self))
	}
}

impl<'de> Deserialize<'de> for SolanaOffchainMessageVersion {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		match u8::deserialize(deserializer)? {
			1 => Ok(SolanaOffchainMessageVersion::V1),
			other => {
				Err(serde::de::Error::custom(format!(
					"unknown offchain message version `{other}`"
				)))
			}
		}
	}
}

/// The app-facing input for `solana:signOffchainMessage`.
///
/// The message is text, not bytes: the wallet encodes it as UTF-8 and
/// constructs the full preamble itself, which is what makes the
/// signature verifiable against the specification rather than against a
/// wallet-chosen encoding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct SolanaSignOffchainMessageProps {
	/// The SRFC-3 message format version to construct.
	#[builder(setter(into))]
	pub message_version: SolanaOffchainMessageVersion,
	/// UTF-8 message body the wallet wraps in the canonical preamble.
	#[builder(setter(into))]
	pub message: String,
	/// Public keys (32 raw bytes each) that must sign; the wallet sorts
	/// and de-duplicates them into the canonical preamble. Must be
	/// non-empty and include the signing account's public key.
	#[builder(setter(into))]
	pub required_signers: Vec<Vec<u8>>,
}

/// The per-message output of a `solana:signOffchainMessage` call.
pub trait SolanaSignOffchainMessageOutput: SolanaSignatureOutput {
	/// The full preamble and body bytes the wallet constructed and
	/// signed, returned verbatim so a verifier can check the signature
	/// without rebuilding the preamble.
	fn signed_offchain_message(&self) -> Vec<u8>;

	/// Optional signature algorithm marker; absent means Ed25519.
	fn signature_type(&self) -> Option<String>;
}

/// Signs offchain messages through the wallet's SRFC-3 implementation.
///
/// Apps use this instead of [`crate::WalletSolanaSignMessage`] whenever
/// the signature must be verifiable without wallet tooling, because the
/// preamble binds the message to its signers and to the offchain domain.
#[async_trait(?Send)]
pub trait WalletSolanaSignOffchainMessage {
	/// The wallet-specific output carrying the signed preamble and
	/// signature.
	type Output: SolanaSignOffchainMessageOutput;

	/// Sign one offchain message with the connected account.
	///
	/// # Errors
	///
	/// Fails with [`crate::WalletError::WalletAccount`] when no account
	/// is attached, and forwards the wallet's rejection otherwise —
	/// including when the account's public key is missing from the
	/// required signers.
	#[allow(clippy::manual_async_fn)]
	async fn sign_offchain_message(
		&self,
		props: SolanaSignOffchainMessageProps,
	) -> WalletResult<Self::Output>;

	/// Sign a batch in one wallet round trip, so the user approves one
	/// prompt for every message.
	///
	/// # Errors
	///
	/// Fails with [`crate::WalletError::InvalidArguments`] for an empty
	/// batch.
	#[allow(clippy::manual_async_fn)]
	async fn sign_offchain_messages(
		&self,
		props: Vec<SolanaSignOffchainMessageProps>,
	) -> WalletResult<Vec<Self::Output>>;
}
