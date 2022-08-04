use std::{marker::PhantomData, rc::Rc};

use crate::{
    deref_optional::{BorrowWrapped, ProjectedDeref, ProjectedDerefMut},
    listeners::Listeners,
    mapper::Mapper,
    optional::IsOptional,
};
pub trait EdgeTrait {
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
    fn invalidate_here_outside(self: &Rc<Self>);
    fn invalidate_up_inside(self: &Rc<Self>);
}

pub struct Edge<E, M, Y>
where
    E: EdgeTrait,
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
    E: EdgeTrait,
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

impl<E, M, Y> EdgeTrait for Edge<E, M, Y>
where
    E: EdgeTrait,
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
    fn invalidate_here_outside(self: &Rc<Self>) {
        self.listeners.invalidate_outside();
    }
    fn invalidate_up_inside(self: &Rc<Self>) {
        self.parent.invalidate_up_inside();
        self.listeners.invalidate_inside();
    }
}
