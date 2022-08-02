use std::{marker::PhantomData, rc::Rc};

use crate::{
    edge::{Edge, EdgeTrait},
    in_enum::InEnumYes,
    mapper::Mapper,
    projectable::{Projectable, ProjectedPart},
    projection::Projection,
};

#[allow(non_snake_case)]
pub struct POption<T, E>
where
    T: Projectable<Edge<E, MapperOption<T>, InEnumYes>>,
    E: EdgeTrait<Data = Option<T>>,
{
    pub Some: ProjectedPart<T, Edge<E, MapperOption<T>, InEnumYes>>,
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
impl<T, E> Projection for POption<T, E>
where
    E: EdgeTrait<Data = Option<T>>,
    T: Projectable<Edge<E, MapperOption<T>, InEnumYes>>,
{
    type Edge = E;

    fn new(edge: Rc<E>) -> Self {
        Self {
            Some: Projection::new(Rc::new(Edge::new(edge.clone(), MapperOption(PhantomData)))),
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
impl<T, E> Projectable<E> for Option<T>
where
    E: EdgeTrait<Data = Option<T>>,
    T: Projectable<Edge<E, MapperOption<T>, InEnumYes>>,
{
    type Projection = POption<T, E>;
}
