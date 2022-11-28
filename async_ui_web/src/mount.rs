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

/** Mount the given element future in a node.
 *
 * This will spawn the element future, causing it to render UI.
 * Everything rendered by the element future will be inside the node.
 */
pub fn mount_at<F: IntoFuture + 'static>(root: F, node: Node) {
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

/** Mount the given future in the page's <body>.
 *
 */
pub fn mount<F: IntoFuture + 'static>(root: F) {
    let node = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap();
    mount_at(root, node.into())
}
