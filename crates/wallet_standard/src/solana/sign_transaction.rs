use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use solana_commitment_config::CommitmentLevel;
use solana_transaction::Transaction;
use solana_transaction::versioned::VersionedTransaction;
use typed_builder::TypedBuilder;

use crate::WalletResult;

/// Feature identifier for the Solana sign-transaction feature.
///
/// Some wallets deliberately omit this feature and offer only
/// [`SOLANA_SIGN_AND_SEND_TRANSACTION`], because handing back a signed
/// transaction lets the app — not the wallet — control broadcasting.
pub const SOLANA_SIGN_TRANSACTION: &str = "solana:signTransaction";

/// The result of a `solana:signTransaction` call.
///
/// The transaction itself is returned rather than bare signatures so
/// multisig, program, and meta-transaction wallets can hand back a
/// modified transaction.
pub trait SolanaSignTransactionOutput {
	/// Signed, serialized transaction, as raw bytes.
	/// Returning a transaction rather than signatures allows multisig wallets,
	/// program wallets, and other wallets that use meta-transactions to return
	/// a modified, signed transaction.
	fn signed_transaction_bytes(&self) -> Vec<u8>;
	/// The signed transaction parsed from the wire bytes.
	///
	/// # Errors
	///
	/// Returns [`WalletError::WalletSignTransaction`] when the wallet
	/// returned bytes that are neither a versioned nor a legacy
	/// transaction, which is how corrupt or foreign payloads surface.
	fn signed_transaction(&self) -> WalletResult<VersionedTransaction>;
}

impl SolanaSignTransactionOutput for VersionedTransaction {
	fn signed_transaction_bytes(&self) -> Vec<u8> {
		bincode::serialize(self).unwrap()
	}

	/// The signed transaction parsed from the wire bytes.
	///
	/// # Errors
	///
	/// Returns [`WalletError::WalletSignTransaction`] when the wallet
	/// returned bytes that are neither a versioned nor a legacy
	/// transaction, which is how corrupt or foreign payloads surface.
	fn signed_transaction(&self) -> WalletResult<VersionedTransaction> {
		Ok(self.clone())
	}
}

impl SolanaSignTransactionOutput for Transaction {
	fn signed_transaction_bytes(&self) -> Vec<u8> {
		let versioned_transaction = VersionedTransaction::from(self.clone());
		bincode::serialize(&versioned_transaction).unwrap()
	}

	fn signed_transaction(&self) -> WalletResult<VersionedTransaction> {
		Ok(self.clone().into())
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// The wire-format input for `solana:signTransaction`.
///
/// This is the shape that actually crosses to the wallet: the transaction
/// travels as raw bytes because the standard deliberately keeps Rust and
/// JavaScript from sharing transaction types.
pub struct SolanaSignTransactionPropsWithBytes {
	/// The versioned transaction which will be encoded into bytes and sent to
	/// the wallet.
	#[builder(setter(into))]
	pub transaction: Vec<u8>,
	/// Chain to use.
	#[builder(default, setter(into))]
	pub chain: Option<String>,
	#[builder(default, setter(into, strip_option))]
	/// Wallet-side preflight constraints; absent options leave the
	/// wallet's defaults in force.
	pub options: Option<SolanaSignTransactionOptions>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// The app-facing input for `solana:signTransaction`.
///
/// Carries a typed [`VersionedTransaction`] which is serialized to wire
/// bytes before crossing to the wallet; see
/// [`SolanaSignTransactionPropsWithBytes`] for the boundary shape.
pub struct SolanaSignTransactionProps {
	/// The versioned transaction which will be encoded into bytes and sent to
	/// the wallet.
	#[builder(setter(into))]
	pub transaction: VersionedTransaction,
	/// Chain to use.
	#[builder(default, setter(into))]
	pub chain: Option<String>,
	/// Additional options for the transaction.
	#[builder(default, setter(into, strip_option(fallback = options_opt)))]
	pub options: Option<SolanaSignTransactionOptions>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// Constraints a wallet should honour while signing.
///
/// These mirror the RPC's preflight semantics so the wallet can simulate
/// against the same state the app will broadcast into, preventing
/// signatures that are already stale.
pub struct SolanaSignTransactionOptions {
	/// Preflight commitment level.
	#[builder(default, setter(strip_option(fallback = preflight_commitment_opt)))]
	pub preflight_commitment: Option<CommitmentLevel>,
	/// The minimum slot that the request can be evaluated at.
	#[builder(default, setter(strip_option(fallback = min_context_slot_opt)))]
	pub min_context_slot: Option<u64>,
}

#[async_trait(?Send)]
/// Signs transactions through the wallet without broadcasting them.
///
/// Apps use this when they need to aggregate signatures, batch, or relay
/// transactions themselves; wallets that refuse to hand back signed
/// transactions simply do not implement the trait.
pub trait WalletSolanaSignTransaction {
	/// The wallet-specific output type carrying the signed wire bytes.
	type Output: SolanaSignTransactionOutput;

	/// Sign a single transaction with the connected account.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::WalletAccount`] when no account is
	/// attached, and forwards the wallet's rejection otherwise.
	async fn sign_transaction(
		&self,
		props: SolanaSignTransactionProps,
	) -> WalletResult<Self::Output>;
	/// Sign a batch in one wallet round trip.
	///
	/// Batches exist because each crossing of the wasm boundary is a
	/// user-visible prompt; one prompt for N transactions is the
	/// difference between one approval and N.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::InvalidArguments`] for an empty batch,
	/// which would otherwise produce a silent no-op prompt.
	async fn sign_transactions(
		&self,
		inputs: Vec<SolanaSignTransactionProps>,
	) -> WalletResult<Vec<Self::Output>>;
}
