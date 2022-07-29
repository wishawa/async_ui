use std::{future::Future, rc::Rc};

use scoped_tls::ScopedKey;

use crate::vnode::VNode;

pub trait BackendTrait: 'static + Sized {
    type Node: Clone + 'static;
    fn add_child_node(
        parent: &Self::Node,
        child: &Self::Node,
        insert_before_sibling: Option<&Self::Node>,
    );
    fn del_child_node(parent: &Self::Node, child: &Self::Node);
    fn drive_executor<F: Future<Output = ()> + 'static>(fut: F);
    fn initialize();
    fn get_vnode_key() -> &'static ScopedKey<Rc<VNode<Self>>>;
}
