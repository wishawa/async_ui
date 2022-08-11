use std::rc::Rc;

use async_ui_core::{
    mount as core_mount,
    vnode::node_concrete::{ConcreteNodeVNode, RefNode, WithNode},
};
use web_sys::Node;

use crate::{backend::Backend, Fragment};

pub fn mount_at(render: Fragment<'static>, node: Node) {
    let fut = WithNode::new(
        render,
        Rc::new(
            ConcreteNodeVNode::new(
                RefNode::<Backend>::Parent { parent: node },
                Default::default(),
            )
            .into(),
        ),
    );
    core_mount::<Backend, _>(fut)
}

pub fn mount(render: Fragment<'static>) {
    let node = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap();
    mount_at(render, node.into())
}
