use crate::tracked::Tracked;

pub trait Trackable<E> {
    type Tracked: Tracked<Edge = E>;
}
pub type TrackedPart<T, E> = <T as Trackable<E>>::Tracked;
