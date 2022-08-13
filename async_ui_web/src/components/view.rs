use std::future::IntoFuture;

use crate::{window::DOCUMENT, Fragment};

use super::ElementFuture;

pub struct View<'c> {
    pub children: Fragment<'c>,
}

impl<'c> IntoFuture for View<'c> {
    type Output = ();
    type IntoFuture = ElementFuture<Fragment<'c>>;

    fn into_future(self) -> Self::IntoFuture {
        ElementFuture::new(
            self.children,
            DOCUMENT
                .with(|doc| doc.create_element("div").expect("create element failed"))
                .into(),
        )
    }
}
