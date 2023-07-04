use crate::node_up::NodeUpTrait;

pub trait NodeDownTrait<'u, T: ?Sized> {
    /// Mark every descendant as "outside" dirty.
    fn invalidate_downward(&self);
    fn node_up(&self) -> &'u (dyn NodeUpTrait<Data = T> + 'u);
}
