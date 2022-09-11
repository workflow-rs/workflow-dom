// use wasm_bindgen::prelude::*;
use web_sys::{Element, Window, Document};
// use js_sys::{Uint8Array, Array};

pub fn window() -> Window{
    web_sys::window().unwrap()
}

pub fn document() -> Document{
    web_sys::window().unwrap().document().unwrap()
}

pub fn body()->std::result::Result<Element, String>{
    let b = document().query_selector("body").unwrap().ok_or("Unable to get body element".to_string())?;
    Ok(b)
}

