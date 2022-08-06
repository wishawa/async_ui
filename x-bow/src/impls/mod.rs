use std::rc::Rc;

use crate::{edge::EdgeTrait, tracked::TrackedNode};

mod stdlib;
pub struct XBowLeaf<T, E>
where
    E: EdgeTrait<Data = T>,
{
    incoming_edge: Rc<E>,
}

impl<T, E> TrackedNode for XBowLeaf<T, E>
where
    E: EdgeTrait<Data = T>,
{
    type Edge = E;

    fn new(edge: Rc<E>) -> Self {
        Self {
            incoming_edge: edge,
        }
    }
    fn edge(&self) -> &Rc<Self::Edge> {
        &self.incoming_edge
    }
    fn invalidate_outside_down(&self) {
        self.edge().invalidate_outside_here();
    }
}
