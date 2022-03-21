use async_ui_reactive::singlethread::ReactiveRefCell;
use std::borrow::Cow;

use web_sys::HtmlAnchorElement;

use crate::elem::Elem;

impl<'a> Elem<'a, HtmlAnchorElement> {
    pub fn href(self, href: &str) -> Self {
        self.elem.set_href(href);
        self
    }
    pub fn href_reactive(mut self, href: ReactiveRefCell<Cow<'static, str>>) -> Self {
        let node = self.elem.clone();
        self.asyncs.push(Box::pin(async move {
            let mut b = href.borrow();
            loop {
                node.set_href(&*b);
                drop(b);
                b = href.borrow_next().await;
            }
        }));
        self
    }
}
