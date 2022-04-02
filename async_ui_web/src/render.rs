use std::rc::Rc;

use async_ui_core::{
    backend::Spawner,
    control::Control,
    drop_check::PropagateDropScope,
    render::{put_node as base_put_node, render_with_control, NodeGuard, RenderFuture},
};
use async_ui_spawners::web::WebSpawner;
use web_sys::Node;

use crate::{
    backend::WebBackend,
    vnode::{NodeVNode, VNode, VNodeEnum},
    Element,
};

pub fn render_in_node<'e>(children: Vec<Element<'e>>, node: Node) -> RenderFuture<'e, WebBackend> {
    render_with_control(
        children,
        Some(Control::new_with_vnode(VNode(Rc::new(VNodeEnum::from(
            NodeVNode::new(node),
        ))))),
    )
}
pub fn render<'e>(children: Vec<Element<'e>>) -> RenderFuture<'e, WebBackend> {
    render_with_control(children, None)
}
pub fn put_node(node: Node) -> NodeGuard<WebBackend> {
    base_put_node::<WebBackend>(node)
}

pub fn mount_at(root: Element<'static>, node: Node) {
    let fut = PropagateDropScope::new(Box::pin(render_in_node(vec![root], node)));
    let task = WebSpawner::spawn(fut);
    task.detach();
    WebSpawner::schedule_now();
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
