use async_ui_core::{backend::BackendTrait, vnode::VNode};
use scoped_tls::{scoped_thread_local, ScopedKey};
use std::{future::Future, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::Node;

use crate::executor::{schedule, set_executor_future};

pub struct Backend;
impl BackendTrait for Backend {
    type Node = Node;

    fn add_child_node(
        parent: &mut Self::Node,
        child: &mut Self::Node,
        insert_before_sibling: Option<&Self::Node>,
    ) {
        parent
            .insert_before(child, insert_before_sibling)
            .expect_throw("insert failed");
    }

    fn del_child_node(parent: &mut Self::Node, child: &mut Self::Node) {
        parent.remove_child(child).expect_throw("remove failed");
    }
    fn drive_executor<F: Future<Output = ()> + 'static>(fut: F) {
        set_executor_future(Box::new(fut) as _);
        schedule();
    }
    fn initialize() {}

    fn get_vnode_key() -> &'static ScopedKey<Rc<VNode<Self>>> {
        &VNODE
    }
}

scoped_thread_local!(
    static VNODE: Rc<VNode<Backend>>
);
