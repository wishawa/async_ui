use enum_dispatch::enum_dispatch;
mod concrete_node;
pub use concrete_node::ConcreteNodeVNode;
mod pass;
pub use pass::PassVNode;

use crate::{backend::BackendTrait, position::PositionIndex};

#[enum_dispatch]
trait VNodeTrait<B: BackendTrait> {
    fn add_child_node(&self, node: B::Node, position: PositionIndex);
    fn del_child_node(&self, position: PositionIndex);
}

#[enum_dispatch(VNodeTrait<B>)]
pub enum VNode<B: BackendTrait> {
    ConcreteNode(ConcreteNodeVNode<B>),
    Pass(PassVNode<B>),
}
