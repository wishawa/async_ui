use crate::tracked::Tracked;

pub trait Trackable<E> {
    type Tracked: Tracked<Edge = E>;
}
pub type HandlePart<T, E> = <T as Trackable<E>>::Tracked;
