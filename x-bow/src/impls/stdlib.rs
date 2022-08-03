use std::{marker::PhantomData, rc::Rc};

use crate::{
    edge::{Edge, EdgeTrait},
    mapper::Mapper,
    optional::OptionalYes,
    projectable::{Trackable, TrackedPart},
    projection::Tracked,
};

#[allow(non_snake_case)]
pub struct POption<T, E>
where
    T: Trackable<Edge<E, MapperOption<T>, OptionalYes>>,
    E: EdgeTrait<Data = Option<T>>,
{
    pub Some: TrackedPart<T, Edge<E, MapperOption<T>, OptionalYes>>,
    incoming_edge: Rc<E>,
}
pub struct MapperOption<T>(PhantomData<T>);

impl<T> Clone for MapperOption<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<T> Mapper for MapperOption<T> {
    type In = Option<T>;
    type Out = T;
    #[inline]
    fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
        input.as_ref()
    }
    #[inline]
    fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
        input.as_mut()
    }
}
impl<T, E> Tracked for POption<T, E>
where
    E: EdgeTrait<Data = Option<T>>,
    T: Trackable<Edge<E, MapperOption<T>, OptionalYes>>,
{
    type Edge = E;

    fn new(edge: Rc<E>) -> Self {
        Self {
            Some: Tracked::new(Rc::new(Edge::new(edge.clone(), MapperOption(PhantomData)))),
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
impl<T, E> Trackable<E> for Option<T>
where
    E: EdgeTrait<Data = Option<T>>,
    T: Trackable<Edge<E, MapperOption<T>, OptionalYes>>,
{
    type Tracked = POption<T, E>;
}
