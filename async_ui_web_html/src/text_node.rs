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
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "csr")]
            node: async_ui_web_core::window::DOCUMENT.with(|doc| doc.create_text_node("")),
            #[cfg(not(feature = "csr"))]
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
