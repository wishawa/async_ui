use async_ui_reactive::local::Rx;

use crate::WrappedWidget;

impl<'a> WrappedWidget<'a, gtk::Label> {
	pub fn text_reactive<S: AsRef<str>>(mut self, text: &'a Rx<S>) -> Self {
		let node_cpy = self.widget.clone();
		self.asyncs.push(Box::pin(text.for_each(move |content| {
			node_cpy.set_text(content.as_ref());
		})));
		self
	}
}
