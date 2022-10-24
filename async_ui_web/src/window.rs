use web_sys::{Document, Window};

thread_local! {
    pub static WINDOW: Window = web_sys::window().expect("no window");
    pub static DOCUMENT: Document = web_sys::window().expect("no window").document().expect("no document");
}
