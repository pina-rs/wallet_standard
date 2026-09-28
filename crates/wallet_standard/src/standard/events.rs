use crate::WalletResult;

/// Feature identifier for the standard events feature.
///
/// Apps listen through this feature to learn when a wallet's chains,
/// features, or authorized accounts change, so their UI can follow the
/// wallet without re-polling.
pub const STANDARD_EVENTS: &str = "standard:events";

/// The changed subset of a wallet's properties reported by a
/// [`STANDARD_EVENTS`] notification.
///
/// Only fields whose value actually changed are populated; absent fields
/// mean "unchanged", which keeps events cheap for wallets that rarely
/// change.
pub trait StandardEventProperties {
	/// The wallet's feature registry type; deliberately opaque because the
	/// standard only requires apps to receive it, not to introspect it.
	type Features;
	/// The account type the wallet hands back, so apps can render updated
	/// account state without downcasting.
	type WalletAccount;

	/// {@link "@wallet-standard/base".Wallet.chains | Chains} supported by the
	/// Wallet.
	///
	/// The Wallet should only define this field if the value of the property
	/// has changed.
	///
	/// The value must be the **new** value of the property.
	fn chains(&self) -> Option<Vec<String>>;
	/// {@link "@wallet-standard/base".Wallet.features | Features} supported by
	/// the Wallet.
	///
	/// The Wallet should only define this field if the value of the property
	/// has changed.
	///
	/// The value must be the **new** value of the property.
	fn features(&self) -> Option<Self::Features>;
	/// {@link "@wallet-standard/base".Wallet.accounts | Accounts} that the app
	/// is authorized to use.
	///
	/// The Wallet should only define this field if the value of the property
	/// has changed.
	///
	/// The value must be the **new** value of the property.
	fn accounts(&self) -> Option<Vec<Self::WalletAccount>>;
}

/// Subscribes to a connected wallet's change notifications, turning the
/// standard's event callbacks into a disposable handle.
pub trait ConnectedWalletStandardEvents {
	/// The callback shape the wallet invokes for each notification; kept as
	/// an associated type so browser and native implementations can differ.
	type Callback;

	/// Listen for changes to the Wallet's properties.
	fn on(&self, event: impl AsRef<str>, callback: &Self::Callback) -> WalletResult<Box<dyn Fn()>>;
}
