use std::rc::Weak;

use crate::{backend::BackendTrait, position::PositionIndex, VNode};

pub struct PassVNode<B: BackendTrait> {
    parent: Weak<VNode<B>>,
    index: usize,
}
