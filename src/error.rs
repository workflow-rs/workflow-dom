//!
//! Errors return by the [`workflow_dom`](super) module
//! 
use thiserror::Error;
use wasm_bindgen::JsValue;

/// Errors return by the [`workflow_dom`](super) module
#[derive(Error, Debug)]
pub enum Error {
    /// Custom string error
    #[error("{0}")]
    String(String),
    /// Error containing [`wasm_bindgen::JsValue`] value
    #[error("{0:?}")]
    JsValue(JsValue),
    #[error("{0}")]
    RecvError(#[from] workflow_core::channel::RecvError),
}

impl From<String> for Error{
    fn from(v:String)->Self{
        Self::String(v)
    }
}

impl From<&str> for Error{
    fn from(v:&str)->Self{
        Self::String(v.to_string())
    }
}

impl From<JsValue> for Error{
    fn from(v:JsValue)->Self{
        Self::JsValue(v)
    }
}