use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::StreamExt;
use futures::channel::mpsc;
use futures::channel::mpsc::UnboundedReceiver;
use futures::stream::Stream;
use wallet_standard::StandardEventProperties;
use wasm_bindgen::closure::Closure;

use crate::BrowserStandardEventsProperties;
use crate::BrowserWallet;
use crate::BrowserWalletAccountInfo;
use crate::Subscription;

/// The `standard:events` notification that reports property changes.
const CHANGE_EVENT: &str = "change";

/// A stream of the connected account as the wallet reports it, built on
/// the `standard:events` change notification.
///
/// The wallet — not the app — decides when the authorized account changes:
/// the user switches accounts or disconnects inside the wallet extension,
/// and the wallet emits a change notification. Apps that read the account
/// once after connect go stale at that point; polling this stream instead
/// means the account shown is always the account the wallet will actually
/// sign with.
///
/// Each item is the wallet's new first authorized account, or `None` when
/// the wallet reports an empty account list (the app is no longer
/// authorized to use any account). Notifications that do not change the
/// accounts property — chains or features changes — produce no item.
///
/// There is no initial item: render the wallet's current account once and
/// let the stream carry changes from there.
///
/// The stream holds its event subscription internally; dropping the stream
/// unsubscribes.
#[derive(Debug)]
pub struct AccountChanges {
	receiver: UnboundedReceiver<Option<BrowserWalletAccountInfo>>,
	// Held for its drop side effect: dropping it unsubscribes the change
	// listener that feeds the receiver.
	_subscription: Subscription,
}

impl Stream for AccountChanges {
	type Item = Option<BrowserWalletAccountInfo>;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		self.receiver.poll_next_unpin(cx)
	}
}

impl BrowserWallet {
	/// Subscribe to the wallet's change notifications as a stream of the
	/// current account.
	///
	/// # Errors
	///
	/// Returns [`WalletError::UnsupportedFeature`] when the wallet does not
	/// implement `standard:events`.
	///
	/// [`WalletError::UnsupportedFeature`]: wallet_standard::WalletError::UnsupportedFeature
	pub fn account_changes(&self) -> wallet_standard::WalletResult<AccountChanges> {
		let feature = self.wallet.get_feature::<crate::StandardEventsFeature>()?;

		let (sender, receiver) = mpsc::unbounded();
		let closure = Closure::wrap(
			Box::new(move |properties: BrowserStandardEventsProperties| {
				// Only account changes produce an item; a notification about
				// chains or features leaves the account stream silent.
				if let Some(accounts) = properties.accounts() {
					let _ = sender.unbounded_send(accounts.into_iter().next());
				}
			}) as Box<dyn Fn(BrowserStandardEventsProperties)>,
		);

		let dispose = feature.on(CHANGE_EVENT, &closure);

		Ok(AccountChanges {
			receiver,
			_subscription: Subscription::new(Box::new(closure), dispose),
		})
	}

	/// Subscribe to the wallet's raw change notifications with a callback.
	///
	/// The callback receives the change properties exactly as the wallet
	/// reported them, covering chains and features changes that
	/// [`BrowserWallet::account_changes`] deliberately filters out.
	///
	/// # Errors
	///
	/// Returns [`WalletError::UnsupportedFeature`] when the wallet does not
	/// implement `standard:events`.
	///
	/// [`WalletError::UnsupportedFeature`]: wallet_standard::WalletError::UnsupportedFeature
	pub fn on_change_with(
		&self,
		callback: impl Fn(BrowserStandardEventsProperties) + 'static,
	) -> wallet_standard::WalletResult<Subscription> {
		let feature = self.wallet.get_feature::<crate::StandardEventsFeature>()?;

		let closure =
			Closure::wrap(Box::new(callback) as Box<dyn Fn(BrowserStandardEventsProperties)>);
		let dispose = feature.on(CHANGE_EVENT, &closure);

		Ok(Subscription::new(Box::new(closure), dispose))
	}
}

/// A stream of the page's registered wallets, starting from the current
/// registry and re-emitting the full list whenever a wallet registers or
/// unregisters.
///
/// Discovery code otherwise writes the same dance by hand: read
/// [`get_wallets`](crate::get_wallets) once for first paint, subscribe to
/// register and unregister for late arrivals, and keep a merged list. This
/// stream is that dance, so a wallet picker is a fold over one stream.
///
/// The stream holds both event subscriptions internally; dropping it
/// unsubscribes from both.
#[derive(Debug)]
pub struct RegistryWatch {
	receiver: UnboundedReceiver<Vec<crate::BrowserWalletInfo>>,
	// Held for their drop side effect: dropping them unsubscribes the
	// register and unregister listeners that feed the receiver.
	_register_subscription: Subscription,
	_unregister_subscription: Subscription,
}

impl Stream for RegistryWatch {
	type Item = Vec<crate::BrowserWalletInfo>;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		self.receiver.poll_next_unpin(cx)
	}
}

/// Watch the page's wallet registry as a stream of wallet lists.
///
/// The first item is the registry as it stands — including every wallet
/// that registered before the call — and each later item is the full list
/// after a wallet registers or unregisters.
pub fn watch_wallets() -> RegistryWatch {
	let wallets = crate::get_wallets();
	let (sender, receiver) = mpsc::unbounded();

	// Setup is synchronous, so no registration can slip between the
	// initial listing and the listeners below.
	sender
		.unbounded_send(wallets.get())
		.expect("the receiver lives in the returned watch, so sending cannot fail here");

	let register_wallets = wallets.clone();
	let register_sender = sender.clone();
	let register_subscription = wallets.on_register_with(move |_| {
		// The listener receives the one new wallet; apps want the whole
		// list, so re-list the registry.
		let _ = register_sender.unbounded_send(register_wallets.get());
	});

	let unregister_wallets = wallets.clone();
	let unregister_subscription = wallets.on_unregister_with(move |_| {
		let _ = sender.unbounded_send(unregister_wallets.get());
	});

	RegistryWatch {
		receiver,
		_register_subscription: register_subscription,
		_unregister_subscription: unregister_subscription,
	}
}
