use std::{
    future::{pending, Future},
    marker::PhantomPinned,
    pin::Pin,
    rc::Rc,
    task::Poll,
};

use async_ui_spawn::{
    wasm::{start_executor, SpawnedFuture, SpawnedTasksContainer},
    RootSpawnWrappedFuture,
};
use web_sys::Node;

pub use crate::control::node_guard::NodeGuard;
use crate::{
    control::element_control::{ElementControl, ELEMENT_CONTROL},
    control::vnode::{NodeVNode, VNode},
    element::Element,
};

pin_project_lite::pin_project! {
    pub struct RenderFuture<'e> {
        children: Vec<Element<'e>>,
        control: Option<ElementControl>,
        #[pin]
        tasks: SpawnedTasksContainer<'e>,
        _pin: PhantomPinned
    }
}

pub(crate) fn render_with_control(
    children: Vec<Element<'_>>,
    control: Option<ElementControl>,
) -> RenderFuture<'_> {
    RenderFuture {
        tasks: SpawnedTasksContainer::with_capacity(children.len()),
        children,
        control,
        _pin: PhantomPinned,
    }
}
impl<'e> Future for RenderFuture<'e> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        if !this.children.is_empty() {
            let control = this
                .control
                .take()
                .unwrap_or_else(|| ELEMENT_CONTROL.with(Clone::clone));
            if this.children.len() == 1 {
                let mut element = this.children.swap_remove(0);
                element.set_control(control);
                this.tasks
                    .launch_futures([element.to_boxed_future()].into_iter());
            } else {
                this.tasks
                    .launch_futures(this.children.drain(..).enumerate().map(
                        |(idx, mut element)| {
                            element.set_control(control.nest(idx));
                            element.to_boxed_future()
                        },
                    ));
            }
            *this.children = Vec::new();
        }
        Poll::Pending
    }
}

pub fn render(children: Vec<Element<'_>>) -> RenderFuture<'_> {
    render_with_control(children, None)
}

pub fn put_node(node: Node) -> NodeGuard {
    ELEMENT_CONTROL.with(|ctr| ctr.put_node(node))
}

pub fn render_in_node(children: Vec<Element<'_>>, node: Node) -> RenderFuture<'_> {
    let control = ElementControl::new_with_vnode(Rc::new(VNode::from(NodeVNode::new(node))));
    render_with_control(children, Some(control))
}

pub fn mount_at(root: Element<'static>, node: Node) {
    wasm_bindgen_futures::spawn_local(RootSpawnWrappedFuture::new(async move {
        web_sys::console::log_1(&"spawn".into());
        let fut = render_in_node(vec![root], node);
        let fut = SpawnedFuture::new(Box::pin(fut));
        let _task = fut.launch();
        start_executor();
        pending().await
    }));
}

pub fn mount(root: Element<'static>) {
    let node = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap();
    mount_at(root, node.into());
}
