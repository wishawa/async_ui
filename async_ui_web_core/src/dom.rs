pub type Node = web_sys::Node;
pub type Element = web_sys::Element;
pub type HtmlElement = web_sys::HtmlElement;
pub type Text = web_sys::Text;
pub type EventTarget = web_sys::EventTarget;
pub type DocumentFragment = web_sys::DocumentFragment;
pub type Comment = web_sys::Comment;

pub use web_sys as elements;

#[inline]
pub fn marker_node(dbg: &'static str) -> Comment {

    let c = Comment::new().unwrap_throw();
    #[cfg(debug_assertions)]
    {
        c.set_data(dbg);
    }
    c
}
