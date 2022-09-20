## WORKFLOW-DOM

Part of the [WORKFLOW-RS](https://github.com/workflow-rs) application framework.

***

Browser DOM manipulation utilities

Platforms supported: WASM (browser)

## Features

* Dynamic (runtime) injection of JsvaScript modules and CSS data into Browser DOM
* Optionally supplied callback gets invoked upon the successful load.

Combined with [`include_bytes!()`](https://doc.rust-lang.org/std/macro.include_bytes.html) macro this crate can be used to dynamically inject JavaScript and CSS files into the browser environment at runtime.

