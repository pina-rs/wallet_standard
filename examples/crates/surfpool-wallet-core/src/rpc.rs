//! Minimal Solana JSON-RPC client for surfpool, built on `fetch` +
//! `serde_json`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde_json::Value;
use serde_json::json;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

#[derive(Debug, Clone)]
pub struct SurfpoolClient {
	pub rpc_url: String,
}

impl SurfpoolClient {
	#[must_use]
	pub fn new(rpc_url: impl Into<String>) -> Self {
		Self {
			rpc_url: rpc_url.into(),
		}
	}

	async fn call(&self, method: &str, params: Value) -> Result<Value, String> {
		let body =
			json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();

		let headers = web_sys::Headers::new().map_err(stringify_js_error)?;
		headers
			.set("Content-Type", "application/json")
			.map_err(stringify_js_error)?;

		let init = web_sys::RequestInit::new();
		init.set_method("POST");
		init.set_headers(&headers);
		init.set_body(&JsValue::from_str(&body));

		let request = web_sys::Request::new_with_str_and_init(&self.rpc_url, &init)
			.map_err(stringify_js_error)?;

		let window = window().ok_or_else(|| "no window".to_string())?;
		let response = JsFuture::from(window.fetch_with_request(&request))
			.await
			.map_err(stringify_js_error)?;
		let response: web_sys::Response = response
			.dyn_into()
			.map_err(|_| "fetch did not return a Response".to_string())?;
		let text = JsFuture::from(response.text().map_err(stringify_js_error)?)
			.await
			.map_err(stringify_js_error)?
			.as_string()
			.ok_or_else(|| "response text is not a string".to_string())?;

		let value: Value = serde_json::from_str(&text)
			.map_err(|error| format!("invalid JSON-RPC response: {error}"))?;

		if let Some(error) = value.get("error") {
			return Err(error.to_string());
		}

		Ok(value
			.get("result")
			.cloned()
			.ok_or_else(|| format!("missing `result` for `{method}`"))?)
	}

	/// Node / surfnet version, used as a health check.
	pub async fn get_version(&self) -> Result<String, String> {
		let result = self.call("getVersion", json!([])).await?;
		Ok(result.to_string())
	}

	pub async fn get_balance(&self, address: &str) -> Result<u64, String> {
		let result = self
			.call("getBalance", json!([address, {"commitment": "confirmed"}]))
			.await?;
		result
			.get("value")
			.and_then(Value::as_u64)
			.ok_or_else(|| "missing balance value".to_string())
	}

	pub async fn request_airdrop(&self, address: &str, lamports: u64) -> Result<String, String> {
		self.call("requestAirdrop", json!([address, lamports]))
			.await?
			.as_str()
			.map(str::to_string)
			.ok_or_else(|| "airdrop signature is not a string".to_string())
	}

	pub async fn get_latest_blockhash(&self) -> Result<(String, u64), String> {
		let result = self
			.call("getLatestBlockhash", json!([{"commitment": "confirmed"}]))
			.await?;
		let blockhash = result
			.get("value")
			.and_then(|value| value.get("blockhash"))
			.and_then(Value::as_str)
			.ok_or_else(|| "missing blockhash".to_string())?
			.to_string();
		let last_valid_block_height = result
			.get("value")
			.and_then(|value| value.get("lastValidBlockHeight"))
			.and_then(Value::as_u64)
			.unwrap_or_default();
		Ok((blockhash, last_valid_block_height))
	}

	/// Sends base64-encoded wire transaction bytes. Returns the transaction
	/// signature (base58).
	pub async fn send_transaction(&self, transaction_bytes: &[u8]) -> Result<String, String> {
		self.send_raw_base64(&BASE64.encode(transaction_bytes))
			.await
	}

	/// Sends an already base64-encoded wire transaction.
	pub async fn send_raw_base64(&self, encoded: &str) -> Result<String, String> {
		self.call(
			"sendTransaction",
			json!([encoded, {"encoding": "base64", "skipPreflight": false, "preflightCommitment": "processed"}]),
		)
		.await?
		.as_str()
		.map(str::to_string)
		.ok_or_else(|| "transaction signature is not a string".to_string())
	}

	/// Returns the confirmation status of a signature, if the node has seen it.
	pub async fn get_signature_status(&self, signature: &str) -> Result<Option<String>, String> {
		let result = self
			.call("getSignatureStatuses", json!([[signature]]))
			.await?;
		let status = result
			.get("value")
			.and_then(Value::as_array)
			.and_then(|values| values.first())
			.and_then(|value| value.get("confirmationStatus"))
			.and_then(Value::as_str)
			.map(str::to_string);
		Ok(status)
	}

	/// Polls until the signature reaches at least "confirmed", or times out.
	pub async fn confirm_signature(&self, signature: &str) -> Result<String, String> {
		for _ in 0..60 {
			if let Some(status) = self.get_signature_status(signature).await? {
				if status == "confirmed" || status == "finalized" {
					return Ok(status);
				}
			}

			sleep_ms(CONFIRMATION_POLL_INTERVAL_MS.try_into().unwrap()).await;
		}

		Err(format!("signature {signature} did not confirm in time"))
	}
}

/// How long to wait between confirmation polls.
const CONFIRMATION_POLL_INTERVAL_MS: u64 = 250;

async fn sleep_ms(ms: i32) {
	let promise = js_sys::Promise::new(&mut |resolve, _| {
		web_sys::window()
			.expect("window")
			.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
			.expect("set_timeout");
	});
	let _ = JsFuture::from(promise).await;
}

fn stringify_js_error(value: JsValue) -> String {
	value.as_string().unwrap_or_else(|| format!("{value:?}"))
}

#[must_use]
pub fn rpc_url_from_location() -> String {
	let Some(location) = window().and_then(|window| {
		let search = window.location().search().ok()?;
		search
			.strip_prefix('?')
			.and_then(find_rpc_param)
			.map(|rpc| {
				let decoded = url_decode(rpc);
				format!("http://{decoded}")
			})
	}) else {
		return crate::DEFAULT_RPC_URL.to_string();
	};
	location
}

fn find_rpc_param(query: &str) -> Option<&str> {
	query.split('&').find_map(|pair| {
		let (key, value) = pair.split_once('=')?;
		(key == "rpc").then_some(value)
	})
}

fn url_decode(value: &str) -> String {
	js_sys::decode_uri(value)
		.map(|decoded| decoded.as_string().unwrap_or_else(|| value.to_string()))
		.unwrap_or_else(|_| value.to_string())
}
