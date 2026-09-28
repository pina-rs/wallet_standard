#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/readme.md"))]
#![allow(clippy::manual_async_fn)]

pub use browser_wallet::*;
pub use browser_wallet_info::*;
pub use constants::*;
pub use features::*;
pub use types::*;
pub use wallet_standard::*;

mod browser_wallet;
mod browser_wallet_info;
mod constants;
mod features;
mod types;

/// Everything a browser dApp or wallet implementation needs in one glob
/// import, re-exporting the core prelude plus the browser-only JS bridge
/// types.
pub mod prelude {
	pub use wallet_standard::prelude::*;

	pub use super::FeatureFromJs;
}
