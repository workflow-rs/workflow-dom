use wasm_bindgen::JsValue;
// pub type Result<T> = std::result::Result<T, crate::error::Error>;
pub type Result<T> = std::result::Result<T, JsValue>;