use std::rc::Rc;

use crate::{edge::EdgeTrait, projection::Tracked};

mod stdlib;
pub struct TrackedLeaf<T, N>
where
    N: EdgeTrait<Data = T>,
{
    incoming_edge: Rc<N>,
}

impl<T, N> Tracked for TrackedLeaf<T, N>
where
    N: EdgeTrait<Data = T>,
{
    type Edge = N;

    fn new(edge: Rc<N>) -> Self {
        Self {
            incoming_edge: edge,
        }
    }
    fn edge(&self) -> &Rc<Self::Edge> {
        &self.incoming_edge
    }
    fn invalidate_here_down(&self) {
        self.edge().invalidate_here();
    }
}
