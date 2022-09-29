use std::future::IntoFuture;

use crate::{window::DOCUMENT, Fragment};

use super::ElementFuture;

pub struct View<'c, I: IntoIterator<Item = ViewProp<'c>>>(pub I);
pub enum ViewProp<'c> {
    Children(Fragment<'c>),
}

impl<'c, I: IntoIterator<Item = ViewProp<'c>>> IntoFuture for View<'c, I> {
    type Output = ();
    type IntoFuture = ElementFuture<Fragment<'c>>;

    fn into_future(self) -> Self::IntoFuture {
        let mut children = None;
        for prop in self.0 {
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
    }
}
