use js_sys::Object;
use js_sys::Reflect;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

/// Locates a named feature object on a wallet's feature map.
///
/// Feature wrappers stay inert until they find their entry, which lets a
/// wallet advertise only what it supports and lets apps probe capability
/// by type instead of by string matching.
pub trait FeatureFromJs: JsCast + Clone + core::fmt::Debug {
	/// The colon separated name of the feature in the JS object.
	const NAME: &'static str;

	/// Get the wallet feature from the JS Object.
	fn feature_from_js_object(object: &Object) -> Option<Self> {
		let feature = Reflect::get(object, &JsValue::from_str(Self::NAME))
			.ok()?
			.unchecked_into();

		Some(feature)
	}

	/// Extract the feature from an arbitrary JavaScript value that should
	/// be a wallet or feature map; `None` means the feature is absent, not
	/// broken.
	fn feature_from_js_value(value: &JsValue) -> Option<Self> {
		Self::feature_from_js_object(value.dyn_ref()?)
	}
}

macro_rules! impl_feature_from_js {
	($ident:ident, $name:expr) => {
		impl $crate::FeatureFromJs for $ident {
			const NAME: &'static str = $name;
		}
	};
}

pub(crate) use impl_feature_from_js;
