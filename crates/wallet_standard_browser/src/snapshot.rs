use std::collections::BTreeSet;

use wallet_standard::WalletInfo;

use crate::BrowserWalletAccountInfo;
use crate::BrowserWalletInfo;

/// An owned, JavaScript-free copy of a [`BrowserWalletInfo`]'s metadata.
///
/// Every accessor on [`BrowserWalletInfo`] crosses into JavaScript and
/// allocates, so UI code that re-reads the name, chains, or features on each
/// render pays that cost repeatedly for values that only change when the
/// wallet emits a change event. [`BrowserWalletInfo::snapshot`] reads each
/// property once and hands back plain Rust values, so repeated reads,
/// comparisons, and hashes stay on the Rust side of the boundary.
///
/// `Hash` and `PartialEq` are derived over the snapshot's values, which also
/// gives wallets whose JavaScript object identity changes across reads a
/// stable identity for keyed state.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BrowserWalletInfoSnapshot {
	/// {@link `WalletVersion` | Version} of the Wallet Standard implemented
	/// by the wallet.
	pub version: String,
	/// Name of the wallet, as displayed by apps.
	pub name: String,
	/// Icon of the wallet as a data URL.
	pub icon: String,
	/// Chains supported by the wallet.
	pub chains: Vec<String>,
	/// Feature names supported by the wallet. A set, because the order in
	/// which the wallet's feature map happened to list them is not part of
	/// the wallet's identity.
	pub features: BTreeSet<String>,
}

impl BrowserWalletInfo {
	/// Read every wallet property into Rust values in one pass.
	///
	/// Costs one JavaScript getter call per property. Afterwards the
	/// snapshot answers reads, comparisons, and hashes without crossing the
	/// boundary again; refresh it when a `standard:events` change
	/// notification reports the wallet changed.
	pub fn snapshot(&self) -> BrowserWalletInfoSnapshot {
		BrowserWalletInfoSnapshot {
			version: self._version(),
			name: self._name(),
			icon: self._icon(),
			chains: self._chains(),
			features: self.features().into_iter().collect(),
		}
	}
}

/// An owned, JavaScript-free copy of a [`BrowserWalletAccountInfo`].
///
/// Account metadata is read through JavaScript accessors just like wallet
/// metadata, so rendering and comparisons should work from a snapshot
/// instead of re-crossing the boundary per field.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BrowserWalletAccountInfoSnapshot {
	/// Address of the account in the chain's own encoding.
	pub address: String,
	/// Raw public key bytes.
	pub public_key: Vec<u8>,
	/// Chains this account may be used on.
	pub chains: Vec<String>,
	/// Feature names this account may be used with.
	pub features: BTreeSet<String>,
	/// Optional user-facing label for the account.
	pub label: Option<String>,
	/// Optional account icon as a data URL.
	pub icon: Option<String>,
}

impl BrowserWalletAccountInfo {
	/// Read every account property into Rust values in one pass.
	///
	/// Costs one JavaScript getter call per property; afterwards the
	/// snapshot answers reads, comparisons, and hashes without crossing the
	/// boundary again.
	pub fn snapshot(&self) -> BrowserWalletAccountInfoSnapshot {
		BrowserWalletAccountInfoSnapshot {
			address: self._address(),
			public_key: self._public_key(),
			chains: self._chains(),
			features: self._features().into_iter().collect(),
			label: self._label(),
			icon: self._icon(),
		}
	}
}
