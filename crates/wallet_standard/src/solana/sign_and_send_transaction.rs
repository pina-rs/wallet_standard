use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use solana_commitment_config::CommitmentLevel;
use solana_transaction::versioned::VersionedTransaction;
use typed_builder::TypedBuilder;

use super::SolanaSignTransactionOptions;
use crate::SolanaSignatureOutput;
use crate::WalletResult;

/// Feature identifier for the Solana sign-and-send feature.
///
/// This is the safer counterpart to [`SOLANA_SIGN_TRANSACTION`]: the
/// wallet signs and broadcasts itself, so a malicious app never holds a
/// signed transaction it can replay or redirect.
pub const SOLANA_SIGN_AND_SEND_TRANSACTION: &str = "solana:signAndSendTransaction";

/// The result of a `solana:signAndSendTransaction` call: the transaction
/// signature, which is all an app receives when the wallet owns
/// broadcasting.
pub trait SolanaSignAndSendTransactionOutput: SolanaSignatureOutput {}
impl<T> SolanaSignAndSendTransactionOutput for T where T: SolanaSignatureOutput {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// The app-facing input for `solana:signAndSendTransaction`.
///
/// The wallet serializes the transaction to wire bytes and broadcasts it
/// under the app's chain and confirmation constraints.
pub struct SolanaSignAndSendTransactionProps {
	/// The versioned transaction.
	#[builder(setter(into))]
	pub transaction: VersionedTransaction,
	/// Chain to use.
	#[builder(default, setter(into, strip_option(fallback = chain_opt)))]
	pub chain: Option<String>,
	#[builder(default, setter(into, strip_option(fallback = options_opt)))]
	/// Signing and confirmation constraints; absent options leave the
	/// wallet's defaults in force.
	pub options: Option<SolanaSignAndSendTransactionOptions>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// How the wallet should reconcile signing with confirmation.
///
/// These mirror the RPC's send semantics so the wallet can wait for the
/// same confirmation the app expects before handing back the signature.
pub struct SolanaSignAndSendTransactionOptions {
	/// Preflight commitment level.
	#[builder(default, setter(into, strip_option(fallback = preflight_commitment_opt)))]
	pub preflight_commitment: Option<CommitmentLevel>,
	/// The minimum slot that the request can be evaluated at.
	#[builder(default, setter(into, strip_option(fallback = min_context_slot_opt)))]
	pub min_context_slot: Option<u64>,
	/// Mode for signing and sending transactions.
	#[builder(default, setter(into, strip_option(fallback = mode_opt)))]
	pub mode: Option<SolanaSignAndSendTransactionMode>,
	/// Desired commitment level. If provided, confirm the transaction after
	/// sending.
	#[builder(default, setter(into, strip_option(fallback = commitment_opt)))]
	pub commitment: Option<CommitmentLevel>,
	/// Disable transaction verification at the RPC.
	#[builder(default, setter(into, strip_option(fallback = skip_preflight_opt)))]
	pub skip_preflight: Option<bool>,
	/// Maximum number of times for the RPC node to retry sending the
	/// transaction to the leader.
	#[builder(default, setter(into, strip_option(fallback = max_retries_opt)))]
	pub max_retries: Option<u8>,
}

impl From<SolanaSignAndSendTransactionOptions> for SolanaSignTransactionOptions {
	fn from(
		SolanaSignAndSendTransactionOptions {
			preflight_commitment,
			min_context_slot,
			..
		}: SolanaSignAndSendTransactionOptions,
	) -> Self {
		Self {
			preflight_commitment,
			min_context_slot,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Whether the wallet waits for its own confirmation before replying.
pub enum SolanaSignAndSendTransactionMode {
	/// Sign and send the transaction.
	Parallel,
	/// Sign the transaction and return it.
	Serial,
}

#[async_trait(?Send)]
/// Signs and broadcasts transactions in one wallet-controlled step.
///
/// Apps that do not need to aggregate or relay transactions should prefer
/// this over [`WalletSolanaSignTransaction`], because the wallet keeps
/// custody of the signed payload end to end.
pub trait WalletSolanaSignAndSendTransaction {
	/// The wallet-specific output type carrying the broadcast signature.
	type Output: SolanaSignAndSendTransactionOutput;

	/// Sign and send a single transaction with the connected account.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::WalletAccount`] when no account is
	/// attached, and forwards the wallet's rejection otherwise.
	async fn sign_and_send_transaction(
		&self,
		props: SolanaSignAndSendTransactionProps,
	) -> WalletResult<Self::Output>;
	/// Sign and send a batch in one wallet round trip, so the user approves
	/// once instead of per transaction.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::InvalidArguments`] for an empty batch and
	/// [`WalletError::UnsupportedTransactionVersion`] when the wallet
	/// cannot sign one of the transactions.
	async fn sign_and_send_transactions(
		&self,
		inputs: Vec<SolanaSignAndSendTransactionProps>,
	) -> WalletResult<Vec<Self::Output>>;
}
