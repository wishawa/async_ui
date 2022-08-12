use std::{
    cell::{Ref, RefMut},
    marker::PhantomData,
    rc::Rc,
};

use crate::{listeners::Listeners, mapper::Mapper, optional::IsOptional};
pub trait TrackedEdge {
    type Data;
    type Optional: IsOptional;
    fn borrow_edge<'b>(self: &'b Rc<Self>) -> Option<Ref<'b, Self::Data>>;
    fn borrow_edge_mut<'b>(self: &'b Rc<Self>) -> Option<RefMut<'b, Self::Data>>;
    fn invalidate_outside_here(self: &Rc<Self>);
    fn invalidate_inside_up(self: &Rc<Self>);
    fn listeners<'s>(self: &'s Rc<Self>) -> &'s Listeners;
}

pub struct Edge<E, M, Y>
where
    E: TrackedEdge,
    M: Mapper<In = E::Data> + Clone,
    Y: IsOptional,
{
    parent: Rc<E>,
    mapper: M,
    listeners: Listeners,
    _phantom: PhantomData<Y>,
}

impl<E, M, Y> Edge<E, M, Y>
where
    E: TrackedEdge,
    M: Mapper<In = E::Data> + Clone,
    Y: IsOptional,
{
    pub fn new(parent: Rc<E>, mapper: M) -> Self {
        let listeners = Listeners::new();
        Self {
            parent,
            mapper,
            listeners,
            _phantom: PhantomData,
        }
    }
}

impl<E, M, Y> TrackedEdge for Edge<E, M, Y>
where
    E: TrackedEdge,
    M: Mapper<In = E::Data> + Clone,
    Y: IsOptional,
{
    type Data = M::Out;
    type Optional = Y;

    fn borrow_edge<'b>(self: &'b Rc<Self>) -> Option<Ref<'b, Self::Data>> {
        self.parent
            .borrow_edge()
            .and_then(|b| Ref::filter_map(b, |v| self.mapper.map(v)).ok())
    }
    fn borrow_edge_mut<'b>(self: &'b Rc<Self>) -> Option<RefMut<'b, Self::Data>> {
        self.parent
            .borrow_edge_mut()
            .and_then(|b| RefMut::filter_map(b, |v| self.mapper.map_mut(v)).ok())
    }
    fn invalidate_outside_here(self: &Rc<Self>) {
        self.listeners.invalidate_outside();
    }
    fn invalidate_inside_up(self: &Rc<Self>) {
        self.parent.invalidate_inside_up();
        self.listeners.invalidate_inside();
    }
    fn listeners<'s>(self: &'s Rc<Self>) -> &'s Listeners {
        &self.listeners
    }
}
