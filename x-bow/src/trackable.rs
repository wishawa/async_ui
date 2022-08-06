use crate::tracked::TrackedNode;

pub trait Trackable<E> {
    type TrackedNode: TrackedNode<Edge = E>;
}
