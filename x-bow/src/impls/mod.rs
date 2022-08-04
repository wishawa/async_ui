use std::rc::Rc;

use crate::{edge::EdgeTrait, tracked::Tracked};

mod stdlib;
pub struct TrackedLeaf<T, E>
where
    E: EdgeTrait<Data = T>,
{
    incoming_edge: Rc<E>,
}

impl<T, E> Tracked for TrackedLeaf<T, E>
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
    fn invalidate_down_outside(&self) {
        self.edge().invalidate_here_outside();
    }
}
