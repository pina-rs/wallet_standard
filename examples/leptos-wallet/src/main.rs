//! Leptos 0.8 CSR example for `wallet_standard_browser` against a surfpool
//! backend.
//!
//! The page embeds a Wallet Standard wallet implemented in Rust (see
//! `surfpool-wallet-core::wallet`) and then talks to it through the *dApp*
//! side of the crate — the exact same code path an app would use to drive
//! Phantom or any other injected wallet.

use std::cell::RefCell;

use leptos::prelude::*;
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

thread_local! {
	static SESSION: RefCell<Option<BrowserWallet>> = const { RefCell::new(None) };
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

fn append_log(log: &RwSignal<Vec<String>>, message: impl core::fmt::Display) {
	log.update(|entries| {
		const MAX_ENTRIES: usize = 200;
		entries.push(message.to_string());
		let len = entries.len();
		if len > MAX_ENTRIES {
			entries.drain(0..len - MAX_ENTRIES);
		}
	});
}

async fn refresh_balance(balance: &RwSignal<Option<u64>>) {
	let Some(address) = connected_address() else {
		return;
	};

	match client().get_balance(&address).await {
		Ok(lamports) => balance.set(Some(lamports)),
		Err(error) => log::warn(format!("getBalance failed: {error}")),
	}
}

/// Connect through the standard, then show the account and its balance.
async fn connect_flow(
	account_address: RwSignal<String>,
	balance: RwSignal<Option<u64>>,
	log: RwSignal<Vec<String>>,
	busy: RwSignal<bool>,
) {
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
			append_log(&log, format!("connected to {}", account.address()));
			refresh_balance(&balance).await;
		}
		Err(error) => append_log(&log, format!("connect failed: {error}")),
	}

	busy.set(false);
}

/// Disconnect and clear everything the connection showed.
async fn disconnect_flow(
	account_address: RwSignal<String>,
	balance: RwSignal<Option<u64>>,
	log: RwSignal<Vec<String>>,
	busy: RwSignal<bool>,
) {
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
			append_log(&log, "disconnected");
		}
		Err(error) => append_log(&log, format!("disconnect failed: {error}")),
	}

	busy.set(false);
}

/// Ask the RPC node for the connected account's balance.
async fn balance_flow(balance: RwSignal<Option<u64>>, log: RwSignal<Vec<String>>) {
	match connected_address() {
		Some(address) => {
			match client().get_balance(&address).await {
				Ok(lamports) => {
					balance.set(Some(lamports));
					append_log(&log, format!("balance: {lamports} lamports"));
				}
				Err(error) => append_log(&log, format!("balance failed: {error}")),
			}
		}

		None => append_log(&log, "connect a wallet first"),
	}
}

/// Airdrop one SOL to the connected account and wait for confirmation.
async fn airdrop_flow(
	balance: RwSignal<Option<u64>>,
	status: RwSignal<String>,
	log: RwSignal<Vec<String>>,
	busy: RwSignal<bool>,
) {
	let Some(address) = connected_address() else {
		status.set("connect a wallet first".to_string());
		busy.set(false);

		return;
	};

	status.set("requesting airdrop…".to_string());

	match client().request_airdrop(&address, AIRDROP_LAMPORTS).await {
		Ok(signature) => {
			let confirmed = client().confirm_signature(&signature).await;
			status.set(match confirmed {
				Ok(state) => format!("airdrop confirmed ({state})"),
				Err(error) => format!("airdrop sent, confirmation failed: {error}"),
			});

			append_log(&log, format!("airdrop signature: {signature}"));
		}
		Err(error) => {
			status.set(format!("airdrop failed: {error}"));
			append_log(&log, format!("airdrop failed: {error}"));
		}
	}

	refresh_balance(&balance).await;
	busy.set(false);
}

/// Sign the demo message with the wallet and verify it locally.
async fn sign_message_flow(status: RwSignal<String>, busy: RwSignal<bool>) {
	let message = b"hello surfpool, from leptos + wallet_standard".to_vec();

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
			status.set(format!(
				"signature {signature} — local ed25519 verification: {}",
				if verified { "VALID" } else { "INVALID" }
			));
		}
		Err(error) => status.set(format!("sign message failed: {error}")),
	}

	busy.set(false);
}

/// Sign and send the demo transfer with the dApp broadcasting it.
async fn send_flow(balance: RwSignal<Option<u64>>, status: RwSignal<String>, busy: RwSignal<bool>) {
	match send_transfer_app_side().await {
		Ok(signature) => status.set(format!("sent and confirmed: {signature}")),
		Err(error) => status.set(format!("send failed: {error}")),
	}

	refresh_balance(&balance).await;
	busy.set(false);
}

/// Sign and send the demo transfer with the wallet broadcasting it.
async fn wallet_send_flow(
	balance: RwSignal<Option<u64>>,
	status: RwSignal<String>,
	busy: RwSignal<bool>,
) {
	match send_transfer_wallet_side().await {
		Ok(signature) => status.set(format!("wallet broadcast: {signature}")),
		Err(error) => status.set(format!("signAndSend failed: {error}")),
	}

	refresh_balance(&balance).await;
	busy.set(false);
}

#[component]
fn App() -> impl IntoView {
	console_error_panic_hook::set_once();

	let rpc_url = rpc::rpc_url_from_location();

	// Boot the wallet side first so it is registered before the dApp side
	// asks the page for registered wallets.
	if let Err(error) = dev_wallet::register_dev_wallet(rpc_url.clone()) {
		web_sys::console::error_1(&format!("failed to register dev wallet: {error:?}").into());
	}

	let wallet_status = RwSignal::new("initialising…".to_string());
	let account_address = RwSignal::new(String::new());
	let balance = RwSignal::new(None::<u64>);
	let busy = RwSignal::new(false);
	let log = RwSignal::new(Vec::<String>::new());

	let airdrop_status = RwSignal::new("—".to_string());
	let sign_message_status = RwSignal::new("—".to_string());
	let send_status = RwSignal::new("—".to_string());
	let wallet_send_status = RwSignal::new("—".to_string());

	match dev_wallet::discover_dev_wallet() {
		Some(wallet) => {
			SESSION.with(|session| *session.borrow_mut() = Some(wallet));
			wallet_status.set(format!("detected: {}", dev_wallet::DEV_WALLET_NAME));
			append_log(&log, "dev wallet registered and discovered");
		}
		None => wallet_status.set("no Wallet Standard wallet detected".to_string()),
	}

	// Each button spawns its flow; the flows own their busy-clearing so a
	// handler can never leave the UI stuck.
	let on_connect = move |_| {
		busy.set(true);
		wasm_bindgen_futures::spawn_local(connect_flow(account_address, balance, log, busy));
	};

	let on_disconnect = move |_| {
		busy.set(true);
		wasm_bindgen_futures::spawn_local(disconnect_flow(account_address, balance, log, busy));
	};

	let on_balance = move |_| {
		wasm_bindgen_futures::spawn_local(balance_flow(balance, log));
	};

	let on_airdrop = move |_| {
		busy.set(true);
		wasm_bindgen_futures::spawn_local(airdrop_flow(balance, airdrop_status, log, busy));
	};

	let on_sign_message = move |_| {
		busy.set(true);
		wasm_bindgen_futures::spawn_local(sign_message_flow(sign_message_status, busy));
	};

	let on_send = move |_| {
		busy.set(true);
		wasm_bindgen_futures::spawn_local(send_flow(balance, send_status, busy));
	};

	let on_wallet_send = move |_| {
		busy.set(true);
		wasm_bindgen_futures::spawn_local(wallet_send_flow(balance, wallet_send_status, busy));
	};

	let balance_text = move || {
		balance
			.get()
			.map(|lamports| format!("{lamports} lamports ({} SOL)", lamports as f64 / 1e9))
			.unwrap_or_else(|| "unknown".to_string())
	};

	view! {
		<header>
			<h1>"wallet_standard × Leptos × surfpool"</h1>
			<span class="badge">"Leptos 0.8 CSR"</span>
		</header>

		<section>
			<h2>"Wallet Standard"</h2>
			<div class="row">
				<span data-testid="wallet-status" class="status">{wallet_status}</span>
			</div>
			<div class="row" style="margin-top:0.5rem">
				<button
				data-testid="connect"
				prop:disabled=move || !account_address.get().is_empty() || busy.get()
				on:click=on_connect
			>"Connect"</button>
				<button
				data-testid="disconnect"
				prop:disabled=move || account_address.get().is_empty() || busy.get()
				on:click=on_disconnect
			>"Disconnect"</button>
			</div>
			<label style="margin-top:0.75rem">"Connected account"</label>
			<div class="value" data-testid="account-address">{account_address}</div>
		</section>

		<section>
			<h2>"surfpool JSON-RPC"</h2>
			<div class="row">
				<label style="margin:0">"endpoint "</label>
				<code data-testid="rpc-url">{rpc_url.clone()}</code>
			</div>
			<div class="row" style="margin-top:0.6rem">
				<button
				data-testid="airdrop"
				prop:disabled=move || account_address.get().is_empty() || busy.get()
				on:click=on_airdrop
			>"Airdrop 1 SOL"</button>
				<button
				data-testid="balance"
				prop:disabled=move || account_address.get().is_empty()
				on:click=on_balance
			>"Refresh balance"</button>
				<span data-testid="balance-value" class="status">{balance_text}</span>
			</div>
			<div class="status" style="margin-top:0.5rem" data-testid="airdrop-status">{airdrop_status}</div>
		</section>

		<section>
			<h2>"solana:signMessage"</h2>
			<div class="row">
				<button
				data-testid="sign-message"
				prop:disabled=move || account_address.get().is_empty() || busy.get()
				on:click=on_sign_message
			>"Sign \"hello surfpool\""</button>
			</div>
			<div class="status" style="margin-top:0.5rem" data-testid="sign-message-status">{sign_message_status}</div>
		</section>

		<section>
			<h2>"Transfer 0.01 SOL → DEMO_RECIPIENT"</h2>
			<div class="row">
				<button
				data-testid="send-app"
				prop:disabled=move || account_address.get().is_empty() || busy.get()
				on:click=on_send
			>"Sign, dApp sends"</button>
				<button
				data-testid="send-wallet"
				prop:disabled=move || account_address.get().is_empty() || busy.get()
				on:click=on_wallet_send
			>"signAndSendTransaction (wallet sends)"</button>
			</div>
			<div class="status" style="margin-top:0.5rem" data-testid="send-app-status">{send_status}</div>
			<div class="status" data-testid="send-wallet-status">{wallet_send_status}</div>
		</section>

		<section>
			<h2>"Activity"</h2>
			<div data-testid="log" class="value" style="min-height:2rem">
				{move || log.get().into_iter().map(|entry| view! { <div>{entry}</div> }).collect_view()}
			</div>
		</section>
	}
}

async fn build_unsigned_transfer() -> Result<
	(
		wallet_standard_browser::BrowserWallet,
		wallet_standard::SolanaSignTransactionProps,
	),
	String,
> {
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
		.map_err(|error| format!("invalid payer address: {error}"))?;
	let to = DEMO_RECIPIENT
		.parse::<Pubkey>()
		.map_err(|error| format!("invalid recipient: {error}"))?;
	let (blockhash, _) = client().get_latest_blockhash().await?;
	let blockhash = blockhash
		.parse::<Hash>()
		.map_err(|error| format!("invalid blockhash: {error}"))?;
	let transaction = tx::build_transfer(&from, &to, TRANSFER_LAMPORTS, &blockhash);
	Ok((wallet, tx::sign_transaction_props(transaction)))
}

/// `solana:signTransaction` → dApp broadcasts via `sendTransaction` and polls
/// `getSignatureStatuses`.
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

/// `solana:signAndSendTransaction` — the wallet signs and broadcasts itself;
/// the dApp only polls for confirmation.
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

mod log {
	pub fn warn(message: impl core::fmt::Display) {
		web_sys::console::warn_1(&message.to_string().into());
	}
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn start() {
	leptos::mount::mount_to_body(App);
}

// The WASM entry point is `#[wasm_bindgen(start)]`; this stub only exists so
// the bin target stays valid for non-wasm hosts.
/// The example airdrop: one SOL, enough to pay for every transfer the demo
/// runs and the fees between them.
const AIRDROP_LAMPORTS: u64 = 1_000_000_000;

/// The demo transfer: a hundredth of the airdrop, so one airdrop funds many runs.
const TRANSFER_LAMPORTS: u64 = 10_000_000;

fn main() {}
