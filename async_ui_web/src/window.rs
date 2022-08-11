use web_sys::{Document, Window};

thread_local! {
    pub(crate) static WINDOW: Window = web_sys::window().expect("no window");
    pub(crate) static DOCUMENT: Document = web_sys::window().expect("no window").document().expect("no document");
}
