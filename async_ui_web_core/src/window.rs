use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Document, Window};

thread_local! {
    pub static WINDOW: Window = web_sys::window().unwrap_throw();
    pub static DOCUMENT: Document = web_sys::window().unwrap_throw().document().unwrap_throw();
}
