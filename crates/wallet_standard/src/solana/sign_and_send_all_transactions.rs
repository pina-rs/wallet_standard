use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

use crate::SolanaSignAndSendTransactionProps;
use crate::SolanaSignatureOutput;
use crate::WalletError;
use crate::WalletResult;

/// Feature identifier for the Solana sign-and-send-all feature.
///
/// The batch counterpart to [`crate::SOLANA_SIGN_AND_SEND_TRANSACTION`]:
/// one wallet prompt drives a whole batch, and each transaction settles
/// independently — a failure in one is reported for that transaction
/// alone instead of failing the entire request.
pub const SOLANA_SIGN_AND_SEND_ALL_TRANSACTIONS: &str = "solana:signAndSendAllTransactions";

/// The result of one transaction in a `solana:signAndSendAllTransactions`
/// batch: either its signature or the reason it alone failed.
///
/// This mirrors the standard's `PromiseSettledResult` per element. The
/// batch call itself only fails when the wallet rejects the request
/// before attempting anything.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
	tag = "status",
	rename_all = "camelCase",
	rename_all_fields = "camelCase"
)]
pub enum WalletSettled<T> {
	/// The transaction was signed and sent; carries the feature output.
	Fulfilled {
		/// The signing output for this transaction.
		value: T,
	},
	/// This transaction alone failed; the rest of the batch is unaffected.
	Rejected {
		/// Why the wallet refused or failed this transaction.
		reason: WalletError,
	},
}

/// The per-transaction output of a `solana:signAndSendAllTransactions`
/// call, identical in shape to the single-shot feature: the broadcast
/// signature, which is all an app receives when the wallet owns
/// broadcasting.
pub trait SolanaSignAndSendAllTransactionsOutput: SolanaSignatureOutput {}
impl<T> SolanaSignAndSendAllTransactionsOutput for T where T: SolanaSignatureOutput {}

/// How the wallet should interleave signing and sending across the batch.
///
/// `Serial` waits for each transaction to land before signing the next,
/// which the standard provides for apps whose transactions depend on each
/// other's effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SolanaSignAndSendAllTransactionsMode {
	/// Sign and send every transaction concurrently.
	Parallel,
	/// Wait for each transaction to land before signing the next.
	Serial,
}

/// Batch-wide options for `solana:signAndSendAllTransactions`.
///
/// Only the mode is standardized; per-transaction constraints stay on
/// each input's own options so a heterogeneous batch stays expressible.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct SolanaSignAndSendAllTransactionsOptions {
	/// How the wallet interleaves signing and sending across the batch;
	/// absent leaves the wallet's default (parallel) in force.
	#[builder(default, setter(into, strip_option))]
	pub mode: Option<SolanaSignAndSendAllTransactionsMode>,
}

/// Signs and sends a batch of transactions in one wallet round trip.
///
/// Apps should prefer this over calling
/// [`crate::WalletSolanaSignAndSendTransaction::sign_and_send_transactions`]
/// when the batch must survive individual failures: partial success (an
/// airdrop that lands while a transfer is refused) is a first-class
/// result here, not an error that discards the outcomes.
#[async_trait(?Send)]
pub trait WalletSolanaSignAndSendAllTransactions {
	/// The wallet-specific per-transaction output carrying the broadcast
	/// signature.
	type Output: SolanaSignAndSendAllTransactionsOutput;

	/// Sign and send every transaction with the connected account,
	/// settling each independently.
	///
	/// # Errors
	///
	/// Fails only when the wallet rejects the request outright (for
	/// example [`WalletError::WalletAccount`] when no account is
	/// attached); per-transaction failures arrive as
	/// [`WalletSettled::Rejected`] elements of the returned batch.
	#[allow(clippy::manual_async_fn)]
	async fn sign_and_send_all_transactions(
		&self,
		inputs: Vec<SolanaSignAndSendTransactionProps>,
		options: SolanaSignAndSendAllTransactionsOptions,
	) -> WalletResult<Vec<WalletSettled<Self::Output>>>;
}
