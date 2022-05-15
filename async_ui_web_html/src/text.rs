use async_ui_reactive::local::Rx;
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
	pub fn content_reactive<S: AsRef<str> + 'a>(mut self, content: &'a Rx<S>) -> Self {
		let node_cpy = self.elem.clone();
		self.asyncs.push(Box::pin(content.for_each(move |s| {
			node_cpy.set_data(s.as_ref());
		})));
		self
	}
}
