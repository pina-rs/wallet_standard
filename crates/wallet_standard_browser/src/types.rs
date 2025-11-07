use js_sys::Object;
use js_sys::Reflect;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

pub trait FeatureFromJs: JsCast + Clone + core::fmt::Debug {
	/// The colon separated name of the feature in the JS object.
	const NAME: &'static str;

	/// Retrieve the feature stored under the JS property named by `Self::NAME` and convert it to `Self`.
	///
	/// Returns `Some(Self)` when the property exists and can be converted into the implementing type, `None` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// // Assume `MyFeature` implements `FeatureFromJs` and sets `NAME` appropriately.
	/// let obj = js_sys::Object::new();
	/// let js_val = wasm_bindgen::JsValue::from_str("some-value");
	/// js_sys::Reflect::set(&obj, &wasm_bindgen::JsValue::from_str(MyFeature::NAME), &js_val).unwrap();
	///
	/// let feature = MyFeature::feature_from_js_object(&obj);
	/// assert!(feature.is_some());
	/// ```
	fn feature_from_js_object(object: &Object) -> Option<Self> {
		let feature = Reflect::get(object, &JsValue::from_str(Self::NAME))
			.ok()?
			.unchecked_into();

		Some(feature)
	}

	/// Attempts to extract this feature from a `JsValue` by interpreting the value as a JavaScript object.
	///
	/// Returns `Some(Self)` if `value` is an object containing the feature keyed by this type's `NAME`, `None` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// // `MyFeature` must implement `FeatureFromJs`.
	/// let js_value: wasm_bindgen::JsValue = /* obtained from JS */ wasm_bindgen::JsValue::NULL;
	/// let feature = MyFeature::feature_from_js_value(&js_value);
	/// ```
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