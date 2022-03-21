use async_ui_web::{
    manual_apis::{put_node, render_in_node, NodeGuard, RenderFuture},
    Element,
};
use std::{any::Any, future::Future, pin::Pin, task::Poll};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Node};

pin_project_lite::pin_project! {
    pub struct Elem<'a, H: 'a> {
        pub(crate) elem: H,
        pub(crate) asyncs: Vec<Pin<Box<dyn Future<Output = ()> + 'a>>>,
        pub(crate) extras: Vec<Box<dyn Any>>,
        #[pin]
        rendered: Option<Rendered<'a>>,
        children: Vec<Element<'a>>
    }
}
pin_project_lite::pin_project! {
    struct Rendered<'a> {
        #[pin]
        future: RenderFuture<'a>,
        guard: NodeGuard
    }
}

impl<'a, H> Elem<'a, H> {
    pub(crate) fn new(elem: H) -> Self {
        Self {
            elem,
            asyncs: Vec::new(),
            extras: Vec::new(),
            rendered: None,
            children: Vec::new(),
        }
    }
}
impl<'a, H: HtmlTag + 'a> Elem<'a, H> {
    pub(crate) fn create(name: &str) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let elem: H = document.create_element(name).unwrap().dyn_into().unwrap();
        Self::new(elem)
    }
}

pub trait HtmlTag: AsRef<HtmlElement> + Clone + JsCast {}

impl<'a, H> Elem<'a, H>
where
    H: HtmlTag + 'a,
{
    pub fn children(mut self, children: Vec<Element<'a>>) -> Self {
        self.children = children;
        self
    }
}

impl<'a, H> Future for Elem<'a, H>
where
    H: AsRef<Node>,
{
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        if this.rendered.is_none() {
            let node: &Node = this.elem.as_ref();
            let node = node.clone();
            let children = std::mem::take(this.children);
            let future = render_in_node(children, node.clone());
            let guard = put_node(node);
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
