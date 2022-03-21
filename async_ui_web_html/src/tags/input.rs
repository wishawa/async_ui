use web_sys::HtmlInputElement;

use crate::elem::Elem;

impl<'a> Elem<'a, HtmlInputElement> {
    pub fn initial_value<'s>(self, value: &str) -> Self {
        self.elem.set_value(value);
        self
    }
    pub fn input_type<'s>(self, input_type: &str) -> Self {
        self.elem.set_type(input_type);
        self
    }
}
