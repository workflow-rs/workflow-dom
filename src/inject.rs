use js_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Element;
use web_sys::{Url,Blob};
use workflow_log::*;
use crate::result::Result;
// use crate::error::Error;
use crate::utils::*;
// use workflow_wasm::listener::Listener;

pub enum Content<'content> {
    Script(&'content [u8]),
    Module(&'content [u8]),
    Style(&'content [u8])
}

pub fn inject_css(css : &str) -> Result<()> {
    let doc = document();
    let head = doc.get_elements_by_tag_name("head").item(0).ok_or("Unable to locate head element")?;
    let style_el = doc.create_element("style")?;
    style_el.set_inner_html(css);
    head.append_child(&style_el)?;
    Ok(())
}

pub fn inject_blob(name: &str, content: Content) ->  Result<()> {
    inject_blob_with_callback(name, content, None)
}

pub fn inject_script(root:Element, content:&[u8], content_type:&str, load : Option<Closure::<dyn FnMut(web_sys::CustomEvent)->Result<()>>>) -> Result<()> {
    let doc = document();
    let string = String::from_utf8_lossy(content);
    let regex = regex::Regex::new(r"//# sourceMappingURL.*$").unwrap();
    let content = regex.replace(&string, "");

    let args = Array::new_with_length(1);
    args.set(0, unsafe { Uint8Array::view(content.as_bytes()).into() });
    let mut options = web_sys::BlobPropertyBag::new();
    options.type_("application/javascript");
    let blob = Blob::new_with_u8_array_sequence_and_options(&args, &options)?;
    let url = Url::create_object_url_with_blob(&blob)?;

    let script = doc.create_element("script")?;
    if let Some(closure) = load {
        script.add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    script.set_attribute("type",content_type)?;
    script.set_attribute("src", &url)?;
    root.append_child(&script)?;

    Ok(())
}

pub fn inject_blob_with_callback(name : &str, content : Content, load : Option<Closure::<dyn FnMut(web_sys::CustomEvent)->Result<()>>>) -> Result<()> {

    log_trace!("loading {}",name);

    let doc = document();
    let root = {
        let collection = doc.get_elements_by_tag_name("head");
        if collection.length() > 0 {
            collection.item(0).unwrap()
        } else {
            doc.get_elements_by_tag_name("body").item(0).unwrap()
        }
    };

    let mime = js_sys::Object::new();
    js_sys::Reflect::set(&mime, &"type".into(), &JsValue::from_str("text/javascript"))?;
    
    match content {
        Content::Script(content) => {
            inject_script(root, content, "text/javascript", load)?;
        },
        Content::Module(content) => {
            inject_script(root, content, "module", load)?;
        },
        Content::Style(content) => {
            let args = Array::new_with_length(1);
            args.set(0, unsafe { Uint8Array::view(content).into() });
            let blob = Blob::new_with_u8_array_sequence(&args)?;
            let url = Url::create_object_url_with_blob(&blob)?;
        
            let style = doc.create_element("link")?;
            if let Some(closure) = load {
                style.add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())?;
                closure.forget();
            }
            style.set_attribute("type","text/css")?;
            style.set_attribute("rel","stylesheet")?;
            style.set_attribute("href",&url)?;
            root.append_child(&style)?;
        },
    }

    Ok(())
}