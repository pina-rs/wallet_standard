use typed_builder::TypedBuilder;
use wallet_standard::Wallet;

use crate::BrowserWalletAccountInfo;
use crate::BrowserWalletInfo;

#[derive(Clone, Debug, PartialEq, Eq, TypedBuilder)]
/// The dApp-side handle to a JavaScript wallet registered on the page.
///
/// Wrapping a [`BrowserWalletInfo`] in this struct is what turns a
/// discovered JS wallet object into the crate's trait implementations, so
/// the same app code drives Phantom and a Rust test wallet identically.
pub struct BrowserWallet {
	/// The currently selected wallet account.
	#[builder(default, setter(strip_option))]
	pub wallet_account: Option<BrowserWalletAccountInfo>,
	/// The currently selected wallet.
	pub wallet: BrowserWalletInfo,
}

impl Wallet for BrowserWallet {
	type Account = BrowserWalletAccountInfo;
	type Wallet = BrowserWalletInfo;

	fn wallet(&self) -> Self::Wallet {
		self.wallet.clone()
	}

	fn wallet_account(&self) -> Option<Self::Account> {
		self.wallet_account.clone()
	}
}

impl From<BrowserWalletInfo> for BrowserWallet {
	fn from(value: BrowserWalletInfo) -> Self {
		BrowserWallet::builder().wallet(value).build()
	}
}

impl From<&BrowserWalletInfo> for BrowserWallet {
	fn from(value: &BrowserWalletInfo) -> Self {
		value.clone().into()
	}
}
