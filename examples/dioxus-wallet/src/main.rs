//! Dioxus 0.7 web example for `wallet_standard_browser` against a surfpool
//! backend.
//!
//! Mirrors the Leptos example: a Wallet Standard wallet implemented in Rust is
//! registered into the page from WASM, and the dApp side of the crate drives
//! it through connect / signMessage / signTransaction / signAndSendTransaction
//! while surfpool serves as the local chain.

use std::cell::RefCell;

use dioxus::prelude::*;
use solana_message::Hash;
use solana_pubkey::Pubkey;
use surfpool_wallet_core::DEMO_RECIPIENT;
use surfpool_wallet_core::rpc;
use surfpool_wallet_core::tx;
use surfpool_wallet_core::wallet as dev_wallet;
use wallet_standard::SolanaSignTransactionOutput;
use wallet_standard::SolanaSignatureOutput;
use wallet_standard::WalletAccountInfo;
use wallet_standard::WalletSolanaSignAndSendTransaction;
use wallet_standard::WalletSolanaSignTransaction;
use wallet_standard_browser::BrowserWallet;
use wasm_bindgen_futures::spawn_local;

thread_local! {
	static SESSION: RefCell<Option<BrowserWallet>> = const { RefCell::new(None) };
	static BOOTED: RefCell<bool> = const { RefCell::new(false) };
}

fn client() -> rpc::SurfpoolClient {
	rpc::SurfpoolClient::new(rpc::rpc_url_from_location())
}

fn connected_address() -> Option<String> {
	SESSION.with(|session| {
		session
			.borrow()
			.as_ref()
			.and_then(|wallet| wallet.wallet_account.clone())
			.map(|account| account.address())
	})
}

fn append_log(log: &mut Signal<Vec<String>>, message: impl core::fmt::Display) {
	const MAX_ENTRIES: usize = 200;
	let entry = message.to_string();
	log.with_mut(|entries| {
		entries.push(entry.clone());
		let len = entries.len();
		if len > MAX_ENTRIES {
			entries.drain(0..len - MAX_ENTRIES);
		}
	});
}

async fn refresh_balance(balance: &mut Signal<Option<u64>>) {
	let Some(address) = connected_address() else {
		return;
	};
	match client().get_balance(&address).await {
		Ok(lamports) => balance.set(Some(lamports)),
		Err(error) => web_sys::console::warn_1(&format!("getBalance failed: {error}").into()),
	}
}

fn boot_once() -> Option<String> {
	let mut result = None;
	BOOTED.with(|booted| {
		if *booted.borrow() {
			return;
		}
		*booted.borrow_mut() = true;

		let rpc_url = rpc::rpc_url_from_location();
		if let Err(error) = dev_wallet::register_dev_wallet(rpc_url) {
			web_sys::console::error_1(&format!("failed to register dev wallet: {error:?}").into());
		}

		match dev_wallet::discover_dev_wallet() {
			Some(wallet) => {
				SESSION.with(|session| *session.borrow_mut() = Some(wallet));
				result = Some(format!("detected: {}", dev_wallet::DEV_WALLET_NAME));
			}
			None => result = Some("no Wallet Standard wallet detected".to_string()),
		}
	});
	result
}

#[component]
fn App() -> Element {
	console_error_panic_hook::set_once();

	let mut wallet_status = use_signal(|| "initialising…".to_string());
	let mut account_address = use_signal(String::new);
	let mut balance = use_signal(|| None::<u64>);
	let mut busy = use_signal(|| false);
	let mut log = use_signal(Vec::<String>::new);

	let mut airdrop_status = use_signal(|| "—".to_string());
	let mut sign_message_status = use_signal(|| "—".to_string());
	let mut send_status = use_signal(|| "—".to_string());
	let mut wallet_send_status = use_signal(|| "—".to_string());

	if let Some(status) = boot_once() {
		wallet_status.set(status);
		append_log(&mut log, "dev wallet registered and discovered");
	}
	let rpc_url = rpc::rpc_url_from_location();

	// -- connect ------------------------------------------------------------
	let connect = move |_| {
		if busy() {
			return;
		}
		busy.set(true);
		spawn_local(async move {
			let result = async {
				let mut wallet = SESSION
					.with(|session| session.borrow().clone())
					.ok_or_else(|| "no wallet available".to_string())?;
				let account = dev_wallet::connect_dev_wallet(&mut wallet)
					.await
					.map_err(|error| error.to_string())?;
				SESSION.with(|session| *session.borrow_mut() = Some(wallet));
				Ok::<_, String>(account)
			}
			.await;
			match result {
				Ok(account) => {
					account_address.set(account.address());
					let mut log = log;
					append_log(&mut log, format!("connected to {}", account.address()));
					let mut balance = balance;
					refresh_balance(&mut balance).await;
				}
				Err(error) => append_log(&mut log, format!("connect failed: {error}")),
			}
			busy.set(false);
		});
	};

	// -- disconnect ---------------------------------------------------------
	let disconnect = move |_| {
		if busy() {
			return;
		}
		busy.set(true);
		spawn_local(async move {
			let result = async {
				let mut wallet = SESSION
					.with(|session| session.borrow().clone())
					.ok_or_else(|| "no wallet available".to_string())?;
				wallet_standard::WalletStandardDisconnect::disconnect(&mut wallet)
					.await
					.map_err(|error| error.to_string())?;
				SESSION.with(|session| *session.borrow_mut() = Some(wallet));
				Ok::<_, String>(())
			}
			.await;
			match result {
				Ok(()) => {
					account_address.set(String::new());
					balance.set(None);
					let mut log = log;
					append_log(&mut log, "disconnected");
				}
				Err(error) => append_log(&mut log, format!("disconnect failed: {error}")),
			}
			busy.set(false);
		});
	};

	// -- balance ------------------------------------------------------------
	let refresh = move |_| {
		spawn_local(async move {
			match connected_address() {
				Some(address) => {
					match client().get_balance(&address).await {
						Ok(lamports) => {
							balance.set(Some(lamports));
							let mut log = log;
							append_log(&mut log, format!("balance: {lamports} lamports"));
						}
						Err(error) => append_log(&mut log, format!("balance failed: {error}")),
					}
				}
				None => append_log(&mut log, "connect a wallet first"),
			}
		});
	};

	// -- airdrop ------------------------------------------------------------
	let airdrop = move |_| {
		if busy() {
			return;
		}
		busy.set(true);
		spawn_local(async move {
			let Some(address) = connected_address() else {
				airdrop_status.set("connect a wallet first".to_string());
				busy.set(false);
				return;
			};
			airdrop_status.set("requesting airdrop…".to_string());
			match client().request_airdrop(&address, 1_000_000_000).await {
				Ok(signature) => {
					let confirmed = client().confirm_signature(&signature).await;
					airdrop_status.set(match confirmed {
						Ok(status) => format!("airdrop confirmed ({status})"),
						Err(error) => format!("airdrop sent, confirmation failed: {error}"),
					});
					let mut log = log;
					append_log(&mut log, format!("airdrop signature: {signature}"));
				}
				Err(error) => {
					airdrop_status.set(format!("airdrop failed: {error}"));
					let mut log = log;
					append_log(&mut log, format!("airdrop failed: {error}"));
				}
			}
			refresh_balance(&mut balance).await;
			busy.set(false);
		});
	};

	// -- sign message -------------------------------------------------------
	let sign_message = move |_| {
		if busy() {
			return;
		}
		busy.set(true);
		spawn_local(async move {
			let message = b"hello surfpool, from dioxus + wallet_standard".to_vec();
			let result = async {
				let wallet = SESSION
					.with(|session| session.borrow().clone())
					.ok_or_else(|| "no wallet available".to_string())?;
				dev_wallet::sign_and_verify_message(&wallet, &message)
					.await
					.map(|(signature, verified)| (signature.to_string(), verified))
					.map_err(|error| error.to_string())
			}
			.await;
			match result {
				Ok((signature, verified)) => {
					sign_message_status.set(format!(
						"signature {signature} — local ed25519 verification: {}",
						if verified { "VALID" } else { "INVALID" }
					));
				}
				Err(error) => sign_message_status.set(format!("sign message failed: {error}")),
			}
			busy.set(false);
		});
	};

	// -- sign + send (dApp owns broadcasting) -------------------------------
	let send_app = move |_| {
		if busy() {
			return;
		}
		busy.set(true);
		spawn_local(async move {
			match send_transfer_app_side().await {
				Ok(signature) => send_status.set(format!("sent and confirmed: {signature}")),
				Err(error) => send_status.set(format!("send failed: {error}")),
			}
			refresh_balance(&mut balance).await;
			busy.set(false);
		});
	};

	// -- sign & send (wallet owns broadcasting) -----------------------------
	let send_wallet = move |_| {
		if busy() {
			return;
		}
		busy.set(true);
		spawn_local(async move {
			match send_transfer_wallet_side().await {
				Ok(signature) => wallet_send_status.set(format!("wallet broadcast: {signature}")),
				Err(error) => wallet_send_status.set(format!("signAndSend failed: {error}")),
			}
			refresh_balance(&mut balance).await;
			busy.set(false);
		});
	};

	rsx! {
		header {
			h1 { "wallet_standard × Dioxus × surfpool" }
			span { class: "badge", "Dioxus 0.7 web" }
		}

		section {
			h2 { "Wallet Standard" }
			div { class: "row",
				span { "data-testid": "wallet-status", class: "status", "{wallet_status}" }
			}
			div { class: "row", style: "margin-top:0.5rem",
				button {
					"data-testid": "connect",
					disabled: !account_address().is_empty() || busy(),
					onclick: connect,
					"Connect"
				}
				button {
					"data-testid": "disconnect",
					disabled: account_address().is_empty() || busy(),
					onclick: disconnect,
					"Disconnect"
				}
			}
			label { style: "margin-top:0.75rem", "Connected account" }
			div { class: "value", "data-testid": "account-address", "{account_address}" }
		}

		section {
			h2 { "surfpool JSON-RPC" }
			div { class: "row",
				label { style: "margin:0", "endpoint " }
				code { "data-testid": "rpc-url", "{rpc_url}" }
			}
			div { class: "row", style: "margin-top:0.6rem",
				button {
					"data-testid": "airdrop",
					disabled: account_address().is_empty() || busy(),
					onclick: airdrop,
					"Airdrop 1 SOL"
				}
				button {
					"data-testid": "balance",
					disabled: account_address().is_empty(),
					onclick: refresh,
					"Refresh balance"
				}
				span { "data-testid": "balance-value", class: "status", "{balance_display(&balance)}" }
			}
			div { class: "status", style: "margin-top:0.5rem", "data-testid": "airdrop-status", "{airdrop_status}" }
		}

		section {
			h2 { "solana:signMessage" }
			div { class: "row",
				button {
					"data-testid": "sign-message",
					disabled: account_address().is_empty() || busy(),
					onclick: sign_message,
					"Sign \"hello surfpool\""
				}
			}
			div { class: "status", style: "margin-top:0.5rem", "data-testid": "sign-message-status", "{sign_message_status}" }
		}

		section {
			h2 { "Transfer 0.01 SOL → DEMO_RECIPIENT" }
			div { class: "row",
				button {
					"data-testid": "send-app",
					disabled: account_address().is_empty() || busy(),
					onclick: send_app,
					"Sign, dApp sends"
				}
				button {
					"data-testid": "send-wallet",
					disabled: account_address().is_empty() || busy(),
					onclick: send_wallet,
					"signAndSendTransaction (wallet sends)"
				}
			}
			div { class: "status", style: "margin-top:0.5rem", "data-testid": "send-app-status", "{send_status}" }
			div { class: "status", "data-testid": "send-wallet-status", "{wallet_send_status}" }
		}

		section {
			h2 { "Activity" }
			div { "data-testid": "log", class: "value", style: "min-height:2rem",
				for entry in log.iter() {
					div { "{entry}" }
				}
			}
		}
	}
}

fn balance_display(balance: &Signal<Option<u64>>) -> String {
	balance()
		.map(|lamports| format!("{lamports} lamports ({} SOL)", lamports as f64 / 1e9))
		.unwrap_or_else(|| "unknown".to_string())
}

async fn build_unsigned_transfer()
-> Result<(BrowserWallet, wallet_standard::SolanaSignTransactionProps), String> {
	let wallet = SESSION
		.with(|session| session.borrow().clone())
		.ok_or("no wallet available")?;
	let address = wallet
		.wallet_account
		.as_ref()
		.map(|account| account.address())
		.ok_or("wallet not connected")?;
	let from = address
		.parse::<Pubkey>()
		.map_err(|e| format!("invalid payer address: {e}"))?;
	let to = DEMO_RECIPIENT
		.parse::<Pubkey>()
		.map_err(|e| format!("invalid recipient: {e}"))?;
	let (blockhash, _) = client().get_latest_blockhash().await?;
	let blockhash = blockhash
		.parse::<Hash>()
		.map_err(|e| format!("invalid blockhash: {e}"))?;
	let transaction = tx::build_transfer(&from, &to, 10_000_000, &blockhash);
	Ok((wallet, tx::sign_transaction_props(transaction)))
}

async fn send_transfer_app_side() -> Result<String, String> {
	let (wallet, props) = build_unsigned_transfer().await?;
	let output = wallet
		.sign_transaction(props)
		.await
		.map_err(|error| format!("wallet rejected transaction: {error}"))?;
	let signed = output
		.signed_transaction()
		.map_err(|error| format!("invalid signed transaction: {error}"))?;
	let wire = tx::serialize_wire_transaction(&signed);
	let signature = client().send_transaction(&wire).await?;
	let status = client().confirm_signature(&signature).await?;
	Ok(format!("{status}: {signature}"))
}

async fn send_transfer_wallet_side() -> Result<String, String> {
	let (wallet, props) = build_unsigned_transfer().await?;
	let props = tx::sign_and_send_props(props.transaction);
	let output = wallet
		.sign_and_send_transaction(props)
		.await
		.map_err(|error| format!("wallet rejected transaction: {error}"))?;
	let signature = output
		.try_signature()
		.map_err(|error| format!("wallet returned invalid signature: {error}"))?;
	let signature_string = signature.to_string();
	let status = client().confirm_signature(&signature_string).await?;
	Ok(format!("{status}: {signature_string}"))
}

fn main() {
	console_error_panic_hook::set_once();
	dioxus::launch(App);
}
