use crate::projection::Projection;

pub trait Projectable<E> {
    type Projection: Projection<Edge = E>;
}
pub type ProjectPart<T, E> = <T as Projectable<E>>::Projection;
