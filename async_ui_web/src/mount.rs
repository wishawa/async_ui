use std::future::IntoFuture;

use async_ui_core::{
    mount as core_mount,
    vnode::node_concrete::{RefNode, WithConcreteNode},
};
use web_sys::Node;

use crate::backend::Backend;

pub fn mount_at<F: IntoFuture<Output = ()> + 'static>(root: F, node: Node) {
    let fut = WithConcreteNode::new(
        root.into_future(),
        RefNode::<Backend>::Parent { parent: node },
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
