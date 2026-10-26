//! A compile-only probe that links the browser crate's public surface into
//! a single wasm binary, so `setup/perf/wasm-size-gate.sh` can gate the
//! size of the code a real dApp pulls in.
//!
//! The probe never runs. Each referenced API only has to stay reachable
//! from `main` so the linker cannot drop the library code it belongs to;
//! runtime-only concerns (JavaScript being present, promises resolving)
//! are irrelevant here.

use wallet_standard::StandardConnectInput;
use wallet_standard::WalletInfo;
use wallet_standard_browser::BrowserWalletAccountInfo;
use wallet_standard_browser::BrowserWalletAccountInfoProps;
#[cfg(feature = "solana")]
use wallet_standard_browser::SolanaSignAndSendTransactionInput;
use wallet_standard_browser::Wallets;

fn main() {
	let registry: Wallets = wallet_standard_browser::get_wallets();

	for wallet in registry.get() {
		let snapshot = wallet.snapshot();
		let _ = (&snapshot.version, &snapshot.name, &snapshot.chains);
	}

	probe_standard(&registry);
	#[cfg(feature = "solana")]
	probe_solana();
}

/// Touches the standard connect and account surface.
fn probe_standard(registry: &Wallets) {
	let _ = StandardConnectInput::default();

	let props = BrowserWalletAccountInfoProps::builder()
		.address("11111111111111111111111111111111")
		.public_key(vec![0u8; 32])
		.build();

	if let Ok(account) = BrowserWalletAccountInfo::try_new(&props) {
		let snapshot = account.snapshot();
		let _ = (&snapshot.address, &snapshot.public_key);
	}

	let _ = registry.get().first().map(WalletInfo::name);
}

/// Touches the Solana signing surface.
#[cfg(feature = "solana")]
fn probe_solana() {
	let message = solana_message::Message::new(&[], None);
	let transaction = solana_transaction::versioned::VersionedTransaction::from(
		solana_transaction::Transaction::new_unsigned(message),
	);

	let props = wallet_standard::SolanaSignAndSendTransactionProps::builder()
		.transaction(transaction)
		.build();

	let account = BrowserWalletAccountInfoProps::builder()
		.address("11111111111111111111111111111111")
		.public_key(vec![0u8; 32])
		.build();

	if let Ok(account) = BrowserWalletAccountInfo::try_new(&account) {
		let _input = SolanaSignAndSendTransactionInput::builder()
			.account(account)
			.transaction(bincode::serialize(&props.transaction).expect("wire bytes serialize"))
			.build();
	}
}
