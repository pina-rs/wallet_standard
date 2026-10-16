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
		// `is_standard_compatible` reads the feature map once for the
		// standard trio; this reads it once more for the Solana trio,
		// instead of one JavaScript getter call per probed feature.
		let features = self.features_object();

		self.is_standard_compatible()
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
