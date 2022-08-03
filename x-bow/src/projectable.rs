use crate::projection::Tracked;

pub trait Trackable<E> {
    type Projection: Tracked<Edge = E>;
}
pub type TrackedPart<T, E> = <T as Trackable<E>>::Projection;
