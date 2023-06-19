use std::rc::Rc;

use crate::node_up::NodeUpTrait;

pub trait NodeDownTrait<'u, T: ?Sized> {
    /// Mark every descendant as "down" dirty.
    fn invalidate_down(&self);
    fn node_up(&self) -> &Rc<dyn NodeUpTrait<Data = T> + 'u>;
}
