use std::future::IntoFuture;

use crate::Fragment;

use super::{create_element_future, ElementFuture};

pub struct View<'c> {
    pub children: Fragment<'c>,
}

impl<'c> IntoFuture for View<'c> {
    type Output = ();
    type IntoFuture = ElementFuture<Fragment<'c>>;

    fn into_future(self) -> Self::IntoFuture {
        create_element_future(self.children, "div")
    }
}
