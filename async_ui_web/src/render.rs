use std::{future::pending, rc::Rc};

use async_ui_spawn::{singlethread::SpawnedFuture, RootSpawnWrappedFuture};
use web_sys::Node;

use crate::{
    control::element_control::{ElementControl, ELEMENT_CONTROL},
    control::{
        node_guard::NodeGuard,
        vnode::{NodeVNode, VNode},
    },
    element::Element,
};

pub(crate) async fn render_with_control<'e>(
    mut children: Vec<Element<'e>>,
    control: ElementControl,
) {
    if children.len() == 1 {
        let mut child = children.pop().unwrap();
        child.set_control(control);
        let mut fut = SpawnedFuture::new(child.to_boxed_future());
        let _task = unsafe { fut.launch_and_get_task() };
        pending().await
    } else {
        let mut tasks = Vec::with_capacity(children.len());
        for (index, mut child) in children.into_iter().enumerate() {
            child.set_control(control.nest(index));
            let mut fut = SpawnedFuture::new(child.to_boxed_future());
            let task = unsafe { fut.launch_and_get_task() };
            tasks.push(task);
        }
        pending().await
    }
}

pub async fn render(children: Vec<Element<'_>>) {
    let control = ELEMENT_CONTROL.with(Clone::clone);
    render_with_control(children, control).await
}

pub fn put_node(node: Node) -> NodeGuard {
    ELEMENT_CONTROL.with(|ctr| ctr.put_node(node))
}

pub async fn render_in_node(children: Vec<Element<'_>>, node: Node) {
    let control = ElementControl::new_with_vnode(Rc::new(VNode::from(NodeVNode::new(node))));
    render_with_control(children, control).await
}

pub async fn mount(root: Element<'static>, node: Node) {
    let fut = render_in_node(vec![root], node);
    let mut fut = SpawnedFuture::new(Box::pin(fut));
    let _task = RootSpawnWrappedFuture::new(async move {
        let _task = unsafe { fut.launch_and_get_task() };
        pending::<()>().await
    })
    .await;
}
