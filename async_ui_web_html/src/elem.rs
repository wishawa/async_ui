use async_ui_web::{
    manual_apis::{control_from_node, put_node, set_render_control, NodeGuard},
    Render,
};
use std::{future::Future, pin::Pin, task::Poll};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Node};

pin_project_lite::pin_project! {
    pub struct Elem<'a, H: 'a> {
        pub(crate) elem: H,
        pub(crate) asyncs: Vec<Pin<Box<dyn Future<Output = ()> + 'a>>>,
        #[pin]
        rendered: Option<Rendered<'a>>,
        children: Option<Render<'a>>
    }
}
pin_project_lite::pin_project! {
    struct Rendered<'a> {
        #[pin]
        future: Render<'a>,
        guard: NodeGuard
    }
}

impl<'a, H> Elem<'a, H> {
    pub(crate) fn new(elem: H) -> Self {
        Self {
            elem,
            asyncs: Vec::new(),
            rendered: None,
            children: None,
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
    pub fn children(mut self, children: impl Into<Render<'a>>) -> Self {
        self.children = Some(children.into());
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
            let children = std::mem::take(this.children);
            let guard = put_node(node.clone());
            let control = control_from_node(node.clone());
            let mut children = children.unwrap_or_else(|| Render::from(()));
            set_render_control(&mut children, control);
            this.rendered.set(Some(Rendered {
                future: children,
                guard,
            }));
        }
        let rendered = this.rendered.as_pin_mut().unwrap().project();
        let _ = rendered.future.poll(cx);
        for job in this.asyncs {
            let _ = job.as_mut().poll(cx);
        }

        Poll::Pending
    }
}
