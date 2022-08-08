use enum_dispatch::enum_dispatch;
pub mod node_concrete;
pub mod node_context;
pub mod node_pass;
use crate::context::ContextMap;

use crate::{backend::BackendTrait, position::PositionIndex};

use self::{node_concrete::ConcreteNodeVNode, node_context::ContextVNode, node_pass::PassVNode};

#[enum_dispatch]
pub(crate) trait VNodeTrait<B: BackendTrait> {
    fn add_child_node(&self, node: B::Node, position: PositionIndex);
    fn del_child_node(&self, position: PositionIndex);
    fn get_context_map<'s>(&'s self) -> &'s ContextMap;
}

#[enum_dispatch(VNodeTrait<B>)]
pub enum VNode<B: BackendTrait> {
    ConcreteNode(ConcreteNodeVNode<B>),
    Context(ContextVNode<B>),
    Pass(PassVNode<B>),
}
