use std::{marker::PhantomData, rc::Rc};

use crate::{
    deref_optional::{BorrowWrapped, ProjectedDeref, ProjectedDerefMut},
    in_enum::IsInEnum,
    listeners::Listeners,
    mapper::Mapper,
};
pub trait EdgeTrait {
    type Data;
    type BorrowGuard<'b>: ProjectedDeref<Target = Self::Data>
    where
        Self: 'b;
    type BorrowMutGuard<'b>: ProjectedDeref<Target = Self::Data> + ProjectedDerefMut
    where
        Self: 'b;
    type InEnum: IsInEnum;
    fn borrow<'b>(self: &'b Rc<Self>) -> Self::BorrowGuard<'b>;
    fn borrow_mut<'b>(self: &'b Rc<Self>) -> Self::BorrowMutGuard<'b>;
    fn invalidate_here(self: &Rc<Self>);
    fn invalidate_up(self: &Rc<Self>);
}

pub struct Edge<E, M, Y>
where
    E: EdgeTrait,
    M: Mapper<In = E::Data> + Clone,
    Y: IsInEnum,
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
    Y: IsInEnum,
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
    Y: IsInEnum,
{
    type Data = M::Out;
    type BorrowGuard<'b> = BorrowWrapped<E::BorrowGuard<'b>, M>
    where
        Self: 'b;
    type BorrowMutGuard<'b> = BorrowWrapped< E::BorrowMutGuard<'b>, M>
    where
        Self: 'b;
    type InEnum = Y;

    fn borrow<'b>(self: &'b Rc<Self>) -> Self::BorrowGuard<'b> {
        BorrowWrapped::new(self.parent.borrow(), self.mapper.clone())
    }

    fn borrow_mut<'b>(self: &'b Rc<Self>) -> Self::BorrowMutGuard<'b> {
        BorrowWrapped::new(self.parent.borrow_mut(), self.mapper.clone())
    }
    fn invalidate_here(self: &Rc<Self>) {
        self.listeners.invalidate();
    }
    fn invalidate_up(self: &Rc<Self>) {
        self.parent.invalidate_here();
        self.parent.invalidate_up()
    }
}
