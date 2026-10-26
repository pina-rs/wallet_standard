use core::any::Any;
use core::fmt;

use js_sys::Function;
use wasm_bindgen::JsValue;

/// An owned handle that keeps a JavaScript event listener alive and removes
/// it on drop.
///
/// A `wasm-bindgen` [`Closure`](wasm_bindgen::closure::Closure) must stay
/// alive for as long as the page holds its listener, and the page's
/// unsubscribe function must be called once the listener is no longer
/// wanted. Tracking those as two separate values invites two classic bugs:
/// dropping the closure early leaves a listener that silently stops
/// working, and forgetting the unsubscribe leaks it forever. `Subscription`
/// owns both, so dropping it does the only correct thing.
///
/// The same subscription machinery backs the [`AccountChanges`] and
/// [`RegistryWatch`] streams, which hold their subscription internally so a
/// stream that is still being polled cannot lose its listener.
///
/// [`AccountChanges`]: crate::AccountChanges
/// [`RegistryWatch`]: crate::RegistryWatch
pub struct Subscription {
	dispose: Function,
	// The closure must outlive the JavaScript listener; the page keeps a
	// reference to it, so dropping it early would leave a listener whose
	// callback no longer exists. Held for its drop side effect only.
	_closure: Box<dyn Any>,
}

impl Subscription {
	/// Bundle a listener closure and the page's unsubscribe function into
	/// one owned handle.
	pub(crate) fn new(closure: Box<dyn Any>, dispose: Function) -> Self {
		Self {
			dispose,
			_closure: closure,
		}
	}
}

impl Drop for Subscription {
	fn drop(&mut self) {
		let _ = self.dispose.call0(&JsValue::NULL);
	}
}

impl fmt::Debug for Subscription {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Subscription").finish_non_exhaustive()
	}
}
