#![feature(generic_associated_types)]
mod borrow_output;
mod deref_optional;
mod in_enum;
mod listeners;
mod mapper;
mod projection;
mod store;

use in_enum::{InEnumYes, IsInEnum};
use listeners::Listeners;
use projection::Projection;
use std::{marker::PhantomData, rc::Rc};
pub use store::{Projected, Store};

use deref_optional::{BorrowWrapped, ProjectedDeref, ProjectedDerefMut};
use mapper::Mapper;
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
pub trait Projectable<T>: EdgeTrait<Data = T> {
    type Projection: Projection<Edge = Self>;
}
pub type ProjectedPart<T, E> = <E as Projectable<T>>::Projection;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}

mod playground {
    use std::rc::Rc;

    use crate::projection::Projection;
    use crate::{POption, Projected, ProjectedPart};

    use super::{mapper::Mapper, Edge, Projectable, Store};

    use super::EdgeTrait;

    struct MyStruct {
        f1: InnerStruct,
        f2: bool,
    }
    struct InnerStruct {
        i1: bool,
        i2: Option<bool>,
    }

    struct PMyStruct<P>
    where
        P: EdgeTrait<Data = MyStruct>,
    {
        pub f1: PInnerStruct<Edge<P, MapperMyStateTof1, P::InEnum>>,
        incoming_edge: Rc<P>,
    }

    impl<P> Projection for PMyStruct<P>
    where
        P: EdgeTrait<Data = MyStruct>,
    {
        type Edge = P;
        fn new(edge: Rc<P>) -> Self {
            let f1 = Projection::new(Rc::new(Edge::new(edge.clone(), MapperMyStateTof1)));
            Self {
                f1,
                incoming_edge: edge,
            }
        }
        fn edge(&self) -> &Rc<Self::Edge> {
            &self.incoming_edge
        }
        fn invalidate_here_down(&self) {
            self.edge().invalidate_here();
            self.f1.invalidate_here_down();
        }
    }
    impl<E> Projectable<MyStruct> for E
    where
        E: EdgeTrait<Data = MyStruct>,
    {
        type Projection = PMyStruct<E>;
    }

    #[derive(Clone)]
    struct MapperMyStateTof1;
    impl Mapper for MapperMyStateTof1 {
        type In = MyStruct;
        type Out = InnerStruct;
        fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
            Some(&input.f1)
        }
        fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
            Some(&mut input.f1)
        }
    }
    struct PInnerStruct<P>
    where
        P: EdgeTrait<Data = InnerStruct>,
    {
        pub i1: Pbool<Edge<P, MapperInnerStateToi1, P::InEnum>>,
        pub i2: POption<bool, Edge<P, MapperInnerStateToi2, P::InEnum>>,
        incoming_edge: Rc<P>,
    }

    impl<N> Projection for PInnerStruct<N>
    where
        N: EdgeTrait<Data = InnerStruct>,
    {
        type Edge = N;
        fn new(edge: Rc<N>) -> Self {
            let i1 = Projection::new(Rc::new(Edge::new(edge.clone(), MapperInnerStateToi1)));
            let i2 = Projection::new(Rc::new(Edge::new(edge.clone(), MapperInnerStateToi2)));
            Self {
                i1,
                i2,
                incoming_edge: edge,
            }
        }
        fn edge(&self) -> &Rc<Self::Edge> {
            &self.incoming_edge
        }

        fn invalidate_here_down(&self) {
            self.edge().invalidate_here();
            self.i1.invalidate_here_down();
        }
    }
    impl<E> Projectable<InnerStruct> for E
    where
        E: EdgeTrait<Data = InnerStruct>,
    {
        type Projection = PInnerStruct<E>;
    }

    #[derive(Clone)]
    struct MapperInnerStateToi1;
    impl Mapper for MapperInnerStateToi1 {
        type In = InnerStruct;
        type Out = bool;
        fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
            Some(&input.i1)
        }
        fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
            Some(&mut input.i1)
        }
    }
    #[derive(Clone)]
    struct MapperInnerStateToi2;
    impl Mapper for MapperInnerStateToi2 {
        type In = InnerStruct;
        type Out = Option<bool>;
        fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
            Some(&input.i2)
        }
        fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
            Some(&mut input.i2)
        }
    }
    pub struct Pbool<N>
    where
        N: EdgeTrait<Data = bool>,
    {
        incoming_edge: Rc<N>,
    }

    impl<N> Projection for Pbool<N>
    where
        N: EdgeTrait<Data = bool>,
    {
        type Edge = N;

        fn new(edge: Rc<N>) -> Self {
            Self {
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

    impl<E> Projectable<bool> for E
    where
        E: EdgeTrait<Data = bool>,
    {
        type Projection = Pbool<E>;
    }

    fn hello() {
        use crate::projection::{ProjectionExt, ProjectionExtGuaranteed};
        let data = MyStruct {
            f1: InnerStruct {
                i2: Some(true),
                i1: false,
            },
            f2: true,
        };
        let store = Store::new(data);
        let proj = store.project();
        let b = *proj.f1.i1.borrow_opt().unwrap();
        let c = &*proj.f1.borrow_opt().unwrap();
        let b = &*proj.f1.borrow();
        let b = &*proj.f1.i2.borrow();
        let b = &proj.f1.i2.Some;
        // let b = *proj.f1.i2.Some.borrow_opt().unwrap();
        take(&proj);
        // take2(&proj.f1);
        fn take(proj: &Projected<MyStruct>) {
            let b = *proj.f1.i1.borrow_opt().unwrap();
            // take2(&proj.f1);
        }
        fn take2(proj: &ProjectedPart<InnerStruct, impl EdgeTrait<Data = InnerStruct>>) {
            let a = proj.i2.borrow_opt();
        }
    }
}

pub struct POption<T, N>
where
    Edge<N, MapperOption<T>, InEnumYes>: Projectable<T>,
    N: EdgeTrait<Data = Option<T>>,
{
    pub Some: ProjectedPart<T, Edge<N, MapperOption<T>, InEnumYes>>,
    incoming_edge: Rc<N>,
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
impl<T, N> Projection for POption<T, N>
where
    Edge<N, MapperOption<T>, InEnumYes>: Projectable<T>,
    N: EdgeTrait<Data = Option<T>>,
{
    type Edge = N;

    fn new(edge: Rc<N>) -> Self {
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
