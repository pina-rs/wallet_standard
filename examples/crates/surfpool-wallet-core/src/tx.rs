//! Transaction construction helpers for the examples.

use solana_message::Message;
use solana_message::VersionedMessage;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_transaction::versioned::VersionedTransaction;
use wallet_standard::SolanaSignAndSendTransactionOptions;
use wallet_standard::SolanaSignAndSendTransactionProps;
use wallet_standard::SolanaSignTransactionProps;
use wallet_standard::WalletResult;

/// A minimal legacy transfer transaction (no lookup tables, one signature).
#[must_use]
pub fn build_transfer(
	from: &Pubkey,
	to: &Pubkey,
	lamports: u64,
	blockhash: &solana_message::Hash,
) -> VersionedTransaction {
	let instruction = solana_system_interface::instruction::transfer(from, to, lamports);
	VersionedTransaction {
		signatures: vec![Signature::default()],
		message: VersionedMessage::Legacy(Message::new_with_blockhash(
			&[instruction],
			Some(from),
			blockhash,
		)),
	}
}

pub fn sign_transaction_props(transaction: VersionedTransaction) -> SolanaSignTransactionProps {
	SolanaSignTransactionProps::builder()
		.transaction(transaction)
		.chain(Some("solana:devnet".to_string()))
		.build()
}

pub fn sign_and_send_props(transaction: VersionedTransaction) -> SolanaSignAndSendTransactionProps {
	SolanaSignAndSendTransactionProps::builder()
		.transaction(transaction)
		.chain("solana:devnet")
		.options(
			SolanaSignAndSendTransactionOptions::builder()
				.commitment(solana_commitment_config::CommitmentLevel::Confirmed)
				.build(),
		)
		.build()
}

pub fn deserialize_wire_transaction(bytes: &[u8]) -> WalletResult<VersionedTransaction> {
	if let Ok(transaction) = bincode::deserialize::<VersionedTransaction>(bytes) {
		return Ok(transaction);
	}

	// The wallet may hand back a legacy transaction; upgrade it.
	let legacy: solana_transaction::Transaction = bincode::deserialize(bytes)
		.map_err(|_| wallet_standard::WalletError::WalletSignTransaction)?;
	Ok(legacy.into())
}

#[must_use]
pub fn serialize_wire_transaction(transaction: &VersionedTransaction) -> Vec<u8> {
	bincode::serialize(transaction).expect("versioned transactions serialize with bincode")
}
