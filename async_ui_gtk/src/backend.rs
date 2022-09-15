use std::rc::Rc;

use async_ui_core::{backend::BackendTrait, vnode::VNode};
use scoped_tls::scoped_thread_local;

use crate::{executor::set_executor_future, widget::WrappedWidget};

pub struct Backend;
impl BackendTrait for Backend {
    type Node = WrappedWidget;

    fn add_child_node(
        parent: &mut Self::Node,
        child: &mut Self::Node,
        insert_before_sibling: Option<&Self::Node>,
    ) {
        parent.add_child_node(child, insert_before_sibling)
    }

    fn del_child_node(parent: &mut Self::Node, child: &mut Self::Node) {
        parent.del_child_node(child)
    }

    fn drive_executor<F: futures::Future<Output = ()> + 'static>(fut: F) {
        set_executor_future(fut)
    }

    fn initialize() {}

    fn get_vnode_key(
    ) -> &'static scoped_tls::ScopedKey<std::rc::Rc<async_ui_core::vnode::VNode<Self>>> {
        &VNODE_TLS
    }
}
scoped_thread_local!(
    static VNODE_TLS: Rc<VNode<Backend>>
);
