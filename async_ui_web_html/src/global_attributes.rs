use std::borrow::Cow;

use crate::elem::{Elem, HtmlTag};
use async_ui_reactive::singlethread::ReactiveRefCell;
use web_sys::HtmlElement;

impl<'a, H: HtmlTag + 'a> Elem<'a, H> {
    pub fn class(self, classes: Vec<&str>) -> Self {
        let elem: &HtmlElement = self.elem.as_ref();
        elem.set_class_name(&classes.join(" "));
        self
    }
    pub fn class_reactive(mut self, classes: &'a ReactiveRefCell<Vec<Cow<'static, str>>>) -> Self {
        let elem: &HtmlElement = self.elem.as_ref();
        let node = elem.clone();
        self.asyncs.push(Box::pin(async move {
            let mut b = classes.borrow();
            loop {
                node.set_class_name(&b.join(" "));
                drop(b);
                b = classes.borrow_next().await;
            }
        }));
        self
    }
}

impl<'a, H: HtmlTag + 'a> Elem<'a, H> {
    pub fn id(self, id: &str) -> Self {
        let elem: &HtmlElement = self.elem.as_ref();
        elem.set_id(id);
        self
    }
}
