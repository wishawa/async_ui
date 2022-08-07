use std::{marker::PhantomData, rc::Rc};

use crate::{edge::TrackedEdge, tracked::TrackedNode};

mod stdlib;
pub struct XBowLeaf<T, E>
where
    E: TrackedEdge<Data = T>,
{
    _phantom: PhantomData<E>,
}

impl<T, E> TrackedNode for XBowLeaf<T, E>
where
    E: TrackedEdge<Data = T>,
{
    type Edge = E;

    fn new(_edge: Rc<E>) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
    fn invalidate_outside_down(&self) {}
}
