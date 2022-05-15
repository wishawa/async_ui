use async_ui_gtk::{
	manual_apis::{control_from_node, put_node, set_render_control, ContainerHandler, NodeGuard},
	Render,
};
use glib::{Cast, IsA};
use gtk::Widget;
use std::{future::Future, pin::Pin, task::Poll};

pin_project_lite::pin_project! {
	pub struct WrappedWidget<'a, H>
	where H: 'a {
		pub(crate) widget: H,
		pub(crate) asyncs: Vec<Pin<Box<dyn Future<Output = ()> + 'a>>>,
		#[pin]
		rendered: Option<Rendered<'a>>,
		pub(crate) children: Option<(Render<'a>, &'static dyn ContainerHandler)>
	}
}
pin_project_lite::pin_project! {
	struct Rendered<'a> {
		#[pin]
		future: Render<'a>,
		guard: NodeGuard
	}
}

impl<'a, H> WrappedWidget<'a, H> {
	pub(crate) fn new(widget: H) -> Self {
		Self {
			widget,
			asyncs: Vec::new(),
			rendered: None,
			children: None,
		}
	}
}

impl<'a, H> WrappedWidget<'a, H> {
	pub fn visit<F: FnOnce(&H)>(self, func: F) -> Self {
		func(&self.widget);
		self
	}
}

impl<'a, H> Future for WrappedWidget<'a, H>
where
	H: IsA<Widget>,
{
	type Output = ();

	fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		let mut this = self.project();
		if this.rendered.is_none() {
			let widget: Widget = this.widget.clone().upcast();
			let future = if let Some((mut children, handler)) = this.children.take() {
				let control = control_from_node(widget.clone().upcast(), handler);
				set_render_control(&mut children, control);
				children
			} else {
				Render::from(())
			};
			let guard = put_node(widget);
			this.rendered.set(Some(Rendered { future, guard }));
		}
		let rendered = this.rendered.as_pin_mut().unwrap().project();
		let _ = rendered.future.poll(cx);
		for job in this.asyncs {
			let _ = job.as_mut().poll(cx);
		}

		Poll::Pending
	}
}
