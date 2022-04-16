use crate::elem::{Elem, HtmlTag};
use async_ui_reactive::local::Rx;
use web_sys::HtmlElement;

impl<'a, H: HtmlTag + 'a> Elem<'a, H> {
    pub fn class(self, classes: Vec<&str>) -> Self {
        let elem: &HtmlElement = self.elem.as_ref();
        elem.set_class_name(&classes.join(" "));
        self
    }
    pub fn class_reactive<S: AsRef<str> + 'a>(mut self, classes: &'a Rx<Vec<S>>) -> Self {
        let elem: &HtmlElement = self.elem.as_ref();
        let node = elem.clone();
        self.asyncs.push(Box::pin(classes.for_each(move |c| {
            let new_class_str: String = c.iter().fold(String::new(), |mut acc, s| {
                acc.push_str(s.as_ref());
                acc
            });
            node.set_class_name(&new_class_str);
        })));
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
