use std::{future::IntoFuture, rc::Rc};

use async_ui_core::{
    mount as core_mount,
    vnode::{
        node_concrete::{ConcreteNodeVNode, RefNode},
        WithVNode,
    },
};
use web_sys::Node;

use crate::backend::Backend;

pub fn mount_at<F: IntoFuture<Output = ()> + 'static>(root: F, node: Node) {
    let fut = WithVNode::new(
        root.into_future(),
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

pub fn mount<F: IntoFuture<Output = ()> + 'static>(root: F) {
    let node = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap();
    mount_at(root, node.into())
}
