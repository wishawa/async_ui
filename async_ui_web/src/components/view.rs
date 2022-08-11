use std::future::IntoFuture;

use crate::Render;

use super::{create_element_future, ElementFuture};

#[derive(Default)]
pub struct View<'c> {
    pub children: Render<'c>,
}

impl<'c> IntoFuture for View<'c> {
    type Output = ();
    type IntoFuture = ElementFuture<Render<'c>>;

    fn into_future(self) -> Self::IntoFuture {
        create_element_future(self.children, "div")
    }
}
