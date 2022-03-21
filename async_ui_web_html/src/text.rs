use std::borrow::Cow;

use async_ui_reactive::singlethread::ReactiveRefCell;
use web_sys::Text;

use crate::elem::Elem;

pub fn text<'a>() -> Elem<'a, Text> {
    let elem = Text::new().expect("text creation failed");
    Elem::new(elem)
}
impl<'a> Elem<'a, Text> {
    pub fn content<'x>(self, content: &'x str) -> Self {
        self.elem.set_data(content);
        self
    }
    pub fn content_reactive(mut self, content: &'a ReactiveRefCell<Cow<'static, str>>) -> Self {
        let node_cpy = self.elem.clone();
        self.asyncs.push(Box::pin(async move {
            let mut ctn = content.borrow();
            loop {
                node_cpy.set_data(&*ctn);
                drop(ctn);
                ctn = content.borrow_next().await;
            }
        }));
        self
    }
}
