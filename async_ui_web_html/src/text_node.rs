use std::{
    future::{pending, Pending},
    ops::Deref,
};

use async_ui_web_core::{dom, ContainerNodeFuture};

/// An HTML text node.
pub struct Text {
    pub node: dom::Text,
}

impl Text {
    #[cfg(feature = "csr")]
    pub fn new() -> Self {
        use async_ui_web_core::window::DOCUMENT;
        Self {
            node: DOCUMENT.with(|doc| doc.create_text_node("")),
        }
    }
    #[cfg(feature = "ssr")]
    pub fn new() -> Self {
        Self {
            node: dom::create_ssr_text("")
        }
    }
    pub fn render(&self) -> ContainerNodeFuture<Pending<()>> {
        ContainerNodeFuture::new(pending(), self.node.clone().into())
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Text {
    type Target = dom::Text;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}
