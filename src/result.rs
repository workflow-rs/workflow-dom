//! Result type used by the [`workflow_dom`](super) module
use wasm_bindgen::JsValue;

/// Result type used by the [`workflow_dom`](super) module
pub type Result<T> = std::result::Result<T, JsValue>;