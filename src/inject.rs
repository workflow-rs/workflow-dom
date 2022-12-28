//!
//! DOM injection utilities, allowing injection of `script` and `style` 
//! elements from Rust buffers using [`Blob`](https://developer.mozilla.org/en-US/docs/Web/API/Blob)
//! objects.
//! 
//! This can be used in conjunction with [`include_bytes`] macro to embed 
//! JavaScript scripts, modules and CSS stylesheets directly within WASM 
//! binary.
//! 

use js_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Element;
use web_sys::{Url,Blob};
use crate::result::*;
use crate::utils::*;
use workflow_core::channel::oneshot;

/// The Content enum specifies the type of the content being injected
pub enum Content<'content> {
    /// This data slice represents a JavaScript script 
    Script(&'content [u8]),
    /// This data slice represents a JavaScript module
    Module(&'content [u8]),
    /// This data slice represents a CSS stylesheet
    Style(&'content [u8])
}

/// Inject CSS stylesheed directly into DOM as a 
/// [`<style>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style) 
/// element using [`Element::set_inner_html`]
pub fn inject_css(css : &str) -> Result<()> {
    let doc = document();
    let head = doc.get_elements_by_tag_name("head").item(0).ok_or("Unable to locate head element")?;
    let style_el = doc.create_element("style")?;
    style_el.set_inner_html(css);
    head.append_child(&style_el)?;
    Ok(())
}

/// Inject a [`Blob`](https://developer.mozilla.org/en-US/docs/Web/API/Blob)
/// into DOM. The `content` argument carries the data buffer and 
/// the content type represented by the [`Content`] struct.
pub fn inject_blob_nowait(content: Content) ->  Result<()> {
    inject_blob_with_callback(content, None)
}

/// Inject a [`Blob`](https://developer.mozilla.org/en-US/docs/Web/API/Blob)
/// into DOM. The `content` argument carries the data buffer and 
/// the content type represented by the [`Content`] struct. This function
/// returns a future that completes upon injection completion.
pub async fn inject_blob(content:Content<'_>) -> Result<()> {
    let (sender, receiver) = oneshot();
    inject_blob_with_callback(content, Some(Closure::<dyn FnMut(web_sys::CustomEvent)->std::result::Result<(), JsValue>>::new(move|notification|->std::result::Result<(), JsValue>{
        sender.try_send(notification).expect("inject_blob_with_callback(): unable to send load notification");
        Ok(())
    })))?;
    let _notification = receiver.recv().await?;
    Ok(())
}

/// Inject script as a [`Blob`](https://developer.mozilla.org/en-US/docs/Web/API/Blob) buffer
/// into DOM. Executes an optional `load` callback when the loading is complete. The load callback
/// receives [`web_sys::CustomEvent`] struct indicating the load result.
pub fn inject_script(root:Element, content:&[u8], content_type:&str, load : Option<Closure::<dyn FnMut(web_sys::CustomEvent)->JsResult<()>>>) -> Result<()> {
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

/// Inject data buffer contained in the [`Content`] struct as a [`Blob`](https://developer.mozilla.org/en-US/docs/Web/API/Blob)
/// into DOM. Executes an optional `load` callback when the loading is complete. The load callback
/// receives [`web_sys::CustomEvent`] struct indicating the load result.
pub fn inject_blob_with_callback(content : Content, load : Option<Closure::<dyn FnMut(web_sys::CustomEvent)->JsResult<()>>>) -> Result<()> {

    let doc = document();
    let root = {
        let collection = doc.get_elements_by_tag_name("head");
        if collection.length() > 0 {
            collection.item(0).unwrap()
        } else {
            doc.get_elements_by_tag_name("body").item(0).unwrap()
        }
    };

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