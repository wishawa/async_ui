use observables::{ObservableAs, ObservableAsExt};

use crate::window::DOCUMENT;

use super::ElementFuture;

pub async fn text<'c>(text: &'c dyn ObservableAs<str>) {
    let node: web_sys::Text = DOCUMENT.with(|doc| doc.create_text_node(""));
    let node_2 = node.clone();
    ElementFuture::new(text.for_each(|t| node_2.set_data(t)), node.into()).await;
}
