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

	let set_busy = move || busy.set(true);
	let clear_busy = move || busy.set(false);

	// -- connect ------------------------------------------------------------
	let connect_log = log;
	let on_connect = move |_| {
		set_busy();
		wasm_bindgen_futures::spawn_local(async move {
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
					append_log(&connect_log, format!("connected to {}", account.address()));
					refresh_balance(&balance).await;
				}
				Err(error) => append_log(&connect_log, format!("connect failed: {error}")),
			}
			clear_busy();
		});
	};

	// -- disconnect ---------------------------------------------------------
	let on_disconnect = move |_| {
		set_busy();
		wasm_bindgen_futures::spawn_local(async move {
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
			clear_busy();
		});
	};

	// -- balance ------------------------------------------------------------
	let balance_log = log;
	let balance_signal = balance;
	let on_balance = move |_| {
		wasm_bindgen_futures::spawn_local(async move {
			match connected_address() {
				Some(address) => {
					match client().get_balance(&address).await {
						Ok(lamports) => {
							balance_signal.set(Some(lamports));
							append_log(&balance_log, format!("balance: {lamports} lamports"));
						}
						Err(error) => append_log(&balance_log, format!("balance failed: {error}")),
					}
				}
				None => append_log(&balance_log, "connect a wallet first"),
			}
		});
	};

	// -- airdrop ------------------------------------------------------------
	let on_airdrop = move |_| {
		set_busy();
		wasm_bindgen_futures::spawn_local(async move {
			let Some(address) = connected_address() else {
				airdrop_status.set("connect a wallet first".to_string());
				clear_busy();
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
					append_log(&log, format!("airdrop signature: {signature}"));
				}
				Err(error) => {
					airdrop_status.set(format!("airdrop failed: {error}"));
					append_log(&log, format!("airdrop failed: {error}"));
				}
			}
			refresh_balance(&balance).await;
			clear_busy();
		});
	};

	// -- sign message -------------------------------------------------------
	let on_sign_message = move |_| {
		set_busy();
		wasm_bindgen_futures::spawn_local(async move {
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
					sign_message_status.set(format!(
						"signature {signature} — local ed25519 verification: {}",
						if verified { "VALID" } else { "INVALID" }
					));
				}
				Err(error) => sign_message_status.set(format!("sign message failed: {error}")),
			}
			clear_busy();
		});
	};

	// -- sign + send (dApp owns broadcasting) -------------------------------
	let on_send = move |_| {
		set_busy();
		wasm_bindgen_futures::spawn_local(async move {
			let result = send_transfer_app_side().await;
			match result {
				Ok(signature) => send_status.set(format!("sent and confirmed: {signature}")),
				Err(error) => send_status.set(format!("send failed: {error}")),
			}
			refresh_balance(&balance).await;
			clear_busy();
		});
	};

	// -- sign & send (wallet owns broadcasting) -----------------------------
	let on_wallet_send = move |_| {
		set_busy();
		wasm_bindgen_futures::spawn_local(async move {
			let result = send_transfer_wallet_side().await;
			match result {
				Ok(signature) => {
					wallet_send_status.set(format!("wallet broadcast: {signature}"));
				}
				Err(error) => wallet_send_status.set(format!("signAndSend failed: {error}")),
			}
			refresh_balance(&balance).await;
			clear_busy();
		});
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
				<button data-testid="connect" prop:disabled=move || !account_address.get().is_empty() || busy.get() on:click=on_connect>"Connect"</button>
				<button data-testid="disconnect" prop:disabled=move || account_address.get().is_empty() || busy.get() on:click=on_disconnect>"Disconnect"</button>
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
				<button data-testid="airdrop" prop:disabled=move || account_address.get().is_empty() || busy.get() on:click=on_airdrop>"Airdrop 1 SOL"</button>
				<button data-testid="balance" prop:disabled=move || account_address.get().is_empty() on:click=on_balance>"Refresh balance"</button>
				<span data-testid="balance-value" class="status">{balance_text}</span>
			</div>
			<div class="status" style="margin-top:0.5rem" data-testid="airdrop-status">{airdrop_status}</div>
		</section>

		<section>
			<h2>"solana:signMessage"</h2>
			<div class="row">
				<button data-testid="sign-message" prop:disabled=move || account_address.get().is_empty() || busy.get() on:click=on_sign_message>"Sign \"hello surfpool\""</button>
			</div>
			<div class="status" style="margin-top:0.5rem" data-testid="sign-message-status">{sign_message_status}</div>
		</section>

		<section>
			<h2>"Transfer 0.01 SOL → DEMO_RECIPIENT"</h2>
			<div class="row">
				<button data-testid="send-app" prop:disabled=move || account_address.get().is_empty() || busy.get() on:click=on_send>"Sign, dApp sends"</button>
				<button data-testid="send-wallet" prop:disabled=move || account_address.get().is_empty() || busy.get() on:click=on_wallet_send>"signAndSendTransaction (wallet sends)"</button>
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
	let transaction = tx::build_transfer(&from, &to, 10_000_000, &blockhash);
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
fn main() {}
