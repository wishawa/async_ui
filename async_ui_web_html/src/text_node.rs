use std::{
    future::{pending, Pending},
    ops::Deref,
};

use async_ui_web_core::{window::DOCUMENT, ContainerNodeFuture};

/// An HTML text node.
pub struct Text {
    pub node: web_sys::Text,
}

impl Text {
    pub fn new() -> Self {
        Self {
            node: DOCUMENT.with(|doc| doc.create_text_node("")),
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
    type Target = web_sys::Text;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}
