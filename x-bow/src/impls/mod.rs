use std::rc::Rc;

use crate::{edge::EdgeTrait, projection::Projection};

mod stdlib;
pub struct ProjectedLeaf<T, N>
where
    N: EdgeTrait<Data = T>,
{
    incoming_edge: Rc<N>,
}

impl<T, N> Projection for ProjectedLeaf<T, N>
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
