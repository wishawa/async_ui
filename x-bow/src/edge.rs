use std::{marker::PhantomData, rc::Rc};

use crate::{
    deref_optional::{BorrowWrapped, ProjectedDeref, ProjectedDerefMut},
    listeners::Listeners,
    mapper::Mapper,
    optional::IsOptional,
};
pub trait TrackedEdge {
    type Data;
    type BorrowGuard<'b>: ProjectedDeref<Target = Self::Data>
    where
        Self: 'b;
    type BorrowMutGuard<'b>: ProjectedDeref<Target = Self::Data> + ProjectedDerefMut
    where
        Self: 'b;
    type Optional: IsOptional;
    fn borrow_edge<'b>(self: &'b Rc<Self>) -> Self::BorrowGuard<'b>;
    fn borrow_edge_mut<'b>(self: &'b Rc<Self>) -> Self::BorrowMutGuard<'b>;
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
    type BorrowGuard<'b> = BorrowWrapped<E::BorrowGuard<'b>, M>
    where
        Self: 'b;
    type BorrowMutGuard<'b> = BorrowWrapped< E::BorrowMutGuard<'b>, M>
    where
        Self: 'b;
    type Optional = Y;

    fn borrow_edge<'b>(self: &'b Rc<Self>) -> Self::BorrowGuard<'b> {
        BorrowWrapped::new(self.parent.borrow_edge(), self.mapper.clone())
    }
    fn borrow_edge_mut<'b>(self: &'b Rc<Self>) -> Self::BorrowMutGuard<'b> {
        BorrowWrapped::new(self.parent.borrow_edge_mut(), self.mapper.clone())
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
