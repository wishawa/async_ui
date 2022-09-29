use crate::{window::DOCUMENT, Fragment};

use super::ElementFuture;

pub enum ViewProp<'c> {
    Children(Fragment<'c>),
}
pub async fn view<'c, I: IntoIterator<Item = ViewProp<'c>>>(props: I) {
    let mut children = None;
    for prop in props {
        match prop {
            ViewProp::Children(v) => children = Some(v),
        }
    }
    ElementFuture::new(
        children.unwrap_or_default(),
        DOCUMENT
            .with(|doc| doc.create_element("div").expect("create element failed"))
            .into(),
    )
    .await;
}
