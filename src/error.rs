//!
//! Errors return by the [`workflow_dom`](super) module
//! 

use wasm_bindgen::JsValue;

/// Errors return by the [`workflow_dom`](super) module
pub enum Error{
    /// Custom string error
    Str(String),
    /// Error containing [`wasm_bindgen::JsValue`] value
    JsValue(JsValue)
}

impl From<String> for Error{
    fn from(v:String)->Self{
        Self::Str(v)
    }
}

impl From<JsValue> for Error{
    fn from(v:JsValue)->Self{
        Self::JsValue(v)
    }
}