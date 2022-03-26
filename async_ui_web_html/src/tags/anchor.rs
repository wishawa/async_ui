use async_ui_reactive::Rx;
use web_sys::HtmlAnchorElement;

use crate::elem::Elem;

impl<'a> Elem<'a, HtmlAnchorElement> {
    pub fn href(self, href: &str) -> Self {
        self.elem.set_href(href);
        self
    }
    pub fn href_reactive<S: AsRef<str>>(mut self, href: &'a Rx<S>) -> Self {
        let node = self.elem.clone();
        self.asyncs.push(Box::pin(href.for_each(move |s| {
            node.set_href(s.as_ref());
        })));
        self
    }
}
