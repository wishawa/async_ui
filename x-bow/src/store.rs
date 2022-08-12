use std::{
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    rc::Rc,
};

use crate::{
    edge::{Edge, TrackedEdge},
    listeners::Listeners,
    mapper::Mapper,
    optional::OptionalNo,
    trackable::Trackable,
    tracked::Tracked,
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
pub(crate) type RootEdge<T> = Edge<RootNode<T>, NoOpMapper<T>, OptionalNo>;
pub type Store<T> = Tracked<<T as Trackable<RootEdge<T>>>::TrackedNode>;
pub struct RootNode<T> {
    data: RefCell<T>,
}

pub fn create_store<T>(data: T) -> Store<T>
where
    T: Trackable<RootEdge<T>>,
{
    let s = Rc::new(RootNode {
        data: RefCell::new(data),
    });
    Tracked::create_with_edge(Rc::new(Edge::new(s, NoOpMapper(PhantomData))))
}
impl<T> TrackedEdge for RootNode<T> {
    type Data = T;
    type Optional = OptionalNo;
    fn borrow_edge<'b>(self: &'b Rc<Self>) -> Option<Ref<'b, Self::Data>> {
        Some(self.data.borrow())
    }

    fn borrow_edge_mut<'b>(self: &'b Rc<Self>) -> Option<RefMut<'b, Self::Data>> {
        Some(self.data.borrow_mut())
    }
    fn invalidate_outside_here(self: &Rc<Self>) {
        unreachable!()
    }
    fn invalidate_inside_up(self: &Rc<Self>) {
        // NO-OP
    }
    fn listeners<'s>(self: &'s Rc<Self>) -> &'s Listeners {
        unreachable!()
    }
}
