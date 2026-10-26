use solana_pubkey::Pubkey;
use wallet_standard::WalletAccountInfo;
use wallet_standard::WalletError;
use wallet_standard::WalletResult;
use wallet_standard::WalletSolanaPubkey;

use crate::BrowserWalletAccountInfo;
use crate::BrowserWalletInfo;
use crate::FeatureFromJs;
use crate::SolanaSignAndSendTransactionFeature;
use crate::SolanaSignMessageFeature;
use crate::SolanaSignTransactionFeature;

impl BrowserWalletInfo {
	/// Whether the wallet implements the standard features plus every
	/// Solana signing feature.
	///
	/// Apps with Solana-only requirements use this as their single
	/// admission check when filtering the registry.
	pub fn is_solana_standard_compatible(&self) -> bool {
		// One read of the feature map feeds all six probes; going through
		// the public `is_standard_compatible` would re-enter JavaScript for
		// the same object, and discovery runs this per registered wallet.
		let features = self.features_object();

		Self::standard_features_present(&features)
			&& SolanaSignMessageFeature::feature_from_js_object(&features).is_some()
			&& SolanaSignTransactionFeature::feature_from_js_object(&features).is_some()
			&& SolanaSignAndSendTransactionFeature::feature_from_js_object(&features).is_some()
	}
}

impl WalletSolanaPubkey for BrowserWalletAccountInfo {
	fn try_solana_pubkey(&self) -> WalletResult<Pubkey> {
		Pubkey::try_from(self.public_key()).map_err(|_| WalletError::WalletPublicKey)
	}
}
