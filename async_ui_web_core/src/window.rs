use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Document, Window};

thread_local! {
    /// The window object. Put in a thread local to avoid frequent unwraps.
    pub static WINDOW: Window = web_sys::window().unwrap_throw();
    /// The document object. Put in a thread local to avoid frequent unwraps.
    pub static DOCUMENT: Document = web_sys::window().unwrap_throw().document().unwrap_throw();
}
