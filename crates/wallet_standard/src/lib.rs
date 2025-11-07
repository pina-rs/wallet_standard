#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/readme.md"))]

pub use error::*;
pub use experimental::*;
#[cfg(feature = "solana")]
pub use solana::*;
pub use standard::*;
pub use types::*;

mod error;
mod experimental;
#[cfg(feature = "solana")]
mod solana;
mod standard;
mod types;

pub mod prelude {
	pub use super::ExperimentalDecryptOutput;
	pub use super::ExperimentalEncryptOutput;
	pub use super::IntoWalletError;
	pub use super::StandardConnectOutput;
	pub use super::Wallet;
	pub use super::WalletAccountInfo;
	pub use super::WalletError;
	pub use super::WalletExperimentalDecrypt;
	pub use super::WalletExperimentalEncrypt;
	pub use super::WalletInfo;
	pub use super::WalletResult;
	pub use super::WalletStandard;
	pub use super::WalletStandardConnect;
	pub use super::WalletStandardDisconnect;
	#[cfg(feature = "solana")]
	pub use super::solana::prelude::*;
}

#[cfg(feature = "solana")]
#[cfg(test)]
mod tests {
	use solana_message::Message;
	use solana_signature::Signature;
	use solana_transaction::Transaction;
	use solana_transaction::versioned::VersionedTransaction;

	use super::*;

	#[test]
	fn versioned_transaction_implements_solana_sign_transaction_output() {
		let transaction =
			VersionedTransaction::from(Transaction::new_unsigned(Message::new(&[], None)));
		insta::assert_json_snapshot!(transaction.signed_transaction_bytes());
	}

	#[test]
	fn transaction_implements_solana_sign_transaction_output() {
		let transaction = Transaction::new_unsigned(Message::new(&[], None));
		insta::assert_json_snapshot!(transaction.signed_transaction_bytes());
	}

	#[test]
	fn signature_implements_signature_output() {
		let signature = Signature::default();
		insta::assert_json_snapshot!(signature.try_signature());
	}
}
