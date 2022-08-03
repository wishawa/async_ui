use std::{
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    rc::Rc,
};

use crate::{
    deref_optional::{ProjectedDeref, ProjectedDerefMut},
    edge::{Edge, EdgeTrait},
    in_enum::InEnumNo,
    mapper::Mapper,
    projectable::{Trackable, TrackedPart},
    projection::Tracked,
};
pub struct NoOpMapper<T>(PhantomData<T>);
impl<T> Clone for NoOpMapper<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<T> Mapper for NoOpMapper<T> {
    type In = T;
    type Out = T;
    fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
        Some(input)
    }
    fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
        Some(input)
    }
}
pub(crate) type RootEdge<T> = Edge<Store<T>, NoOpMapper<T>, InEnumNo>;
pub type Projected<T> = TrackedPart<T, RootEdge<T>>;
pub struct Store<T> {
    data: RefCell<T>,
}

impl<T> Store<T>
where
    T: Trackable<RootEdge<T>>,
{
    pub fn new(data: T) -> Rc<Self> {
        Rc::new(Self {
            data: RefCell::new(data),
        })
    }
    pub fn project(self: &Rc<Self>) -> Projected<T> {
        Tracked::new(Rc::new(Edge::new(self.clone(), NoOpMapper(PhantomData))))
    }
}
impl<T> EdgeTrait for Store<T> {
    type Data = T;
    type BorrowGuard<'b> = Ref<'b, T>
    where
        Self: 'b;
    type BorrowMutGuard<'b> = RefMut<'b, T>
    where
        Self: 'b;

    type InEnum = InEnumNo;
    fn borrow<'b>(self: &'b Rc<Self>) -> Self::BorrowGuard<'b> {
        self.data.borrow()
    }

    fn borrow_mut<'b>(self: &'b Rc<Self>) -> Self::BorrowMutGuard<'b> {
        self.data.borrow_mut()
    }
    fn invalidate_here(self: &Rc<Self>) {
        // NO-OP
    }
    fn invalidate_up(self: &Rc<Self>) {
        // NO-OP
    }
}

impl<'b, T> ProjectedDeref for Ref<'b, T> {
    type Target = T;
    fn deref_optional(&self) -> Option<&Self::Target> {
        Some(&*self)
    }
}
impl<'b, T> ProjectedDeref for RefMut<'b, T> {
    type Target = T;
    fn deref_optional(&self) -> Option<&Self::Target> {
        Some(&*self)
    }
}
impl<'b, T> ProjectedDerefMut for RefMut<'b, T> {
    fn deref_mut_optional(&mut self) -> Option<&mut Self::Target> {
        Some(&mut *self)
    }
}
