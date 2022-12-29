use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use workflow_log::*;
use workflow_wasm::callback::*;
use js_sys::{Array,Uint8Array};
use web_sys::{Document,Url,Blob};
use workflow_core::channel::oneshot;
use futures::future::{join_all,BoxFuture,FutureExt};
use workflow_core::lookup::*;
use crate::error::Error;
use crate::result::Result;

pub type Id = u32;
pub type ModuleMap = HashMap<Id,Arc<Module>>;

static mut DOCUMENT_ROOT : Option<web_sys::Element> = None;

pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

pub fn root() -> web_sys::Element {
    unsafe {
        match DOCUMENT_ROOT.as_ref() {
            Some(root) => root.clone(),
            None => {
                let root = {
                    let collection = document().get_elements_by_tag_name("head");
                    if collection.length() > 0 {
                        collection.item(0).unwrap()
                    } else {
                        document().get_elements_by_tag_name("body").item(0).unwrap()
                    }
                };
                DOCUMENT_ROOT = Some(root.clone());
                root
            }
        }
    }
}

#[allow(dead_code)]
pub enum Reference {
    Style,
    Module,
    Script,
    Export,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ModuleStatus {
    Loaded,
    Exists,
    Error,
}

pub struct Module {
    pub url : Mutex<Option<String>>,
    pub id : Id,
    pub ident : &'static str,
    pub content: &'static str,
    pub references: &'static [(Reference, Option<&'static str>, Id)],
    pub is_loaded : AtomicBool,
}

// unsafe impl Send for Module {}
// unsafe impl Sync for Module {}

impl Module {

    pub fn url(&self) -> Option<String> { 
        self.url.lock().unwrap().clone() 
    }

    fn content(&self, ctx: &Context) -> Result<String> {
        let mut text = String::new();

        let mut imports = Vec::new();
        let mut exports = Vec::new();

        for (kind,what,id) in self.references.iter() {
            let module = ctx.get(id).ok_or(format!("unable to lookup module `{}`",self.ident))?;
            let url = module.url().ok_or(format!("[{}] module is not loaded `{}`",self.ident,id))?;
            match kind {
                Reference::Style => {
                    // TODO: import style into the global document
                    log_info!("TODO: loading style {}", what.unwrap())
                },
                Reference::Script => {
                    // TODO: import style into the global document
                    log_info!("TODO: loading style {}", what.unwrap())
                },
                Reference::Module => {
                    match what {
                        Some(detail) => {
                            imports.push(format!("import {} from \"{}\";\n", detail, url));
                        },
                        None => {
                            imports.push(format!("import \"{}\";\n", url));
                        }
                    }
                },
                Reference::Export => {
                    let module = ctx.get(id).ok_or(format!("unable to lookup module `{}`",self.ident))?;
                    let url = module.url().ok_or(format!("[{}] module is not loaded `{}`",self.ident,id))?;
                    exports.push(format!("export {} from \"{}\";\n", what.unwrap(), url));
                }
            }
        }

        let imports = imports.join("\n");
        let exports = exports.join("\n");

        text += &imports;
        text += &self.content;
        text += &exports;

        Ok(text)
    }

    pub fn is_loaded(&self) -> bool {
        self.is_loaded.load(Ordering::SeqCst)
    }

    fn load_deps(self : Arc<Self>, ctx: Arc<Context>) -> BoxFuture<'static, Result<()>> {

        async move {
            let futures = self.references
                .iter()
                .map(|(kind,what,id)| {
                    match kind {
                        Reference::Module | Reference::Script | Reference::Export => {
                            if let Some(module) = ctx.get(id) {
                                if !module.is_loaded.load(Ordering::SeqCst) {
                                    Some(module.load(&ctx))
                                } else {
                                    None
                                }
                            } else {
                                log_error!("Unable to locate module {}", id);
                                None
                            }
                        },
                        Reference::Style => {
                            log_warning!("TODO: import stylesheet: {:?}",what);
                            None
                        }
                    }
                })
                .flatten()
                .collect::<Vec<_>>();
            
            join_all(futures).await;

            Ok(())
        }.boxed()

        // Some(future)

        // Ok(())
    }

    pub async fn load(self : &Arc<Self>, ctx: &Arc<Context>) -> Result<ModuleStatus> {
        ctx.load_module(self).await
    }

    fn create_blob_url_for_script(&self, ctx: &Arc<Context>) -> Result<String> {
        let content = self.content(ctx)?;
        let args = Array::new_with_length(1);
        args.set(0, unsafe { Uint8Array::view(content.as_bytes()).into() });
        let mut options = web_sys::BlobPropertyBag::new();
        // options.type_("module");
        options.type_("application/javascript");
        let blob = Blob::new_with_u8_array_sequence_and_options(&args, &options)?;
        let url = Url::create_object_url_with_blob(&blob)?;
        self.url.lock().unwrap().replace(url.clone());
        Ok(url)
    }

    async fn load_impl(self : &Arc<Self>, ctx: &Arc<Context>) -> Result<ModuleStatus> {

        if self.is_loaded() {
            return Ok(ModuleStatus::Exists);
        }
        
        self.clone().load_deps(ctx.clone()).await?;
        log_info!("load ... {}", self.ident);
        
        let (sender,receiver) = oneshot();
        let url = self.create_blob_url_for_script(ctx)?;
        let ident = self.ident.clone();
        let callback = callback!(move |_event: web_sys::CustomEvent| {
            log_info!("{} ... done", ident);
            // TODO - analyze event
            let status = ModuleStatus::Loaded;
            sender.try_send(status).expect("unable to post load event");
        });
        self.inject_script(&url, &callback)?;
        let status = receiver.recv().await.expect("unable to recv() load event");
        self.is_loaded.store(true, Ordering::SeqCst);
        Ok(status)
    }

    fn inject_script<C>(&self, url : &str, callback : &C) -> Result<()>
    where
        C: AsRef<js_sys::Function>
    {
        let script = document().create_element("script")?;                    
        script.add_event_listener_with_callback("load", callback.as_ref())?;
        let content_type = "module";
        script.set_attribute("module","true")?;
        script.set_attribute("type",content_type)?;
        script.set_attribute("src", url)?;
        root().append_child(&script)?;
        Ok(())
    }

}


pub struct Context {
    pub modules : Arc<ModuleMap>,
    pub lookup_handler : LookupHandler<Id,ModuleStatus,Error>,
}

impl Context {

    pub fn new(modules : ModuleMap) -> Context {
        Context {
            modules : Arc::new(modules),
            lookup_handler: LookupHandler::new(),
        }
    }

    pub fn get<'l>(&'l self, id : &Id) -> Option<&'l Arc<Module>> {
        self.modules.get(id)
    }

    pub async fn load_module(self: &Arc<Self>, module : &Arc<Module>) -> Result<ModuleStatus> {

        if module.is_loaded() {
            Ok(ModuleStatus::Exists)
        } else {
        
            match self.lookup_handler.queue(&module.id).await {
                RequestType::New(receiver) => {
                    let result = module.load_impl(self).await;
                    self.lookup_handler.complete(&module.id, result).await;
                    receiver.recv().await?
                },
                RequestType::Pending(receiver) => {
                    receiver.recv().await?
                }
            }
        }
    }

    pub async fn load_ids(self : &Arc<Self>, list : &[Id]) -> Result<()> {

        let futures = list
            .iter()
            .map(|id| {
                if let Some(module) = self.get(id) {
                    Some(module.load(self))
                } else {
                    log_error!("Unable to locate module {}", id);
                    // TODO: panic
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();
        
        for future in futures {
            match future.await {
                Ok(_event) => {

                },
                Err(err) => {
                    log_error!("{}", err);
                }
            }
        }

        Ok(())
    }
}
