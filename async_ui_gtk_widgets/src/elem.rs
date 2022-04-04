use async_ui_gtk::{
    manual_apis::{put_node, render_in_node, ContainerHandler, NodeGuard, RenderFuture},
    render, Element,
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
        pub(crate) children: Option<(Vec<Element<'a>>, &'static dyn ContainerHandler)>
    }
}
pin_project_lite::pin_project! {
    struct Rendered<'a> {
        #[pin]
        future: RenderFuture<'a>,
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

pub trait Wrappable<'a>: Sized {
    fn wrap(self) -> WrappedWidget<'a, Self>;
}
impl<'a, H: IsA<Widget>> Wrappable<'a> for H {
    fn wrap(self) -> WrappedWidget<'a, Self> {
        WrappedWidget::new(self)
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
            let future = if let Some((children, handler)) = this.children.take() {
                let future = render_in_node(children, widget.clone(), handler);
                future
            } else {
                render(vec![])
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
