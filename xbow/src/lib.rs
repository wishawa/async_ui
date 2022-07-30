#![feature(generic_associated_types)]
mod deref_wrap;
mod mapper;
mod new;

use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use deref_wrap::DerefWrapped;
use mapper::Mapper;
use new::New;
pub trait EdgeTrait {
    type Data;
    type BorrowGuard<'b>: Deref<Target = Self::Data>
    where
        Self: 'b;
    type BorrowMutGuard<'b>: Deref<Target = Self::Data> + DerefMut
    where
        Self: 'b;
    fn borrow<'b>(self: &'b Rc<Self>) -> Self::BorrowGuard<'b>;
    fn borrow_mut<'b>(self: &'b Rc<Self>) -> Self::BorrowMutGuard<'b>;
}
// pub trait EdgeTrait: Clone {
//     type Out;
//     type Parent: NodeTrait;
//     type ParentMapper: Mapper<In = <Self::Parent as NodeTrait>::Data, Out = Self::Out>;
//     fn new_edge(parent: Weak<Self::Parent>) -> Self;
//     // fn map<'s, 'b>(&'s self, parent: &'b <Self::Parent as NodeTrait>::Data) -> &'b Self::Out;
//     // fn map_mut<'s, 'b>(
//     //     &'s self,
//     //     parent: &'b mut <Self::Parent as NodeTrait>::Data,
//     // ) -> &'b mut Self::Out;
//     fn mapper(&self) -> Self::ParentMapper;
// }

pub struct MappingEdge<P, M>
where
    P: EdgeTrait,
    M: Mapper<In = P::Data> + Clone,
{
    parent: Rc<P>,
    mapper: M,
}

impl<P, M> MappingEdge<P, M>
where
    P: EdgeTrait,
    M: Mapper<In = P::Data> + Clone,
{
    pub fn new(parent: Rc<P>, mapper: M) -> Self {
        Self { parent, mapper }
    }
}

impl<P, M> EdgeTrait for MappingEdge<P, M>
where
    P: EdgeTrait,
    M: Mapper<In = P::Data> + Clone,
{
    type Data = M::Out;
    type BorrowGuard<'b> = DerefWrapped<P::BorrowGuard<'b>, M>
    where
        Self: 'b;
    type BorrowMutGuard<'b> = DerefWrapped<P::BorrowMutGuard<'b>, M>
    where
        Self: 'b;

    fn borrow<'b>(self: &'b Rc<Self>) -> Self::BorrowGuard<'b> {
        DerefWrapped::new(self.parent.borrow(), self.mapper.clone())
    }

    fn borrow_mut<'b>(self: &'b Rc<Self>) -> Self::BorrowMutGuard<'b> {
        DerefWrapped::new(self.parent.borrow_mut(), self.mapper.clone())
    }
}
pub trait Projectable<T>: EdgeTrait<Data = T> {
    type Projection: New<Arg = Rc<Self>>;
}
pub type Project<T, E> = <E as Projectable<T>>::Projection;
pub struct Store<T> {
    data: RefCell<T>,
}

impl<T> Store<T>
where
    Store<T>: Projectable<T>,
{
    pub fn new(data: T) -> Rc<Self> {
        Rc::new(Self {
            data: RefCell::new(data),
        })
    }
    pub fn project(self: &Rc<Self>) -> Project<T, Store<T>> {
        New::new(self.clone())
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
    fn borrow<'b>(self: &'b Rc<Self>) -> Self::BorrowGuard<'b> {
        self.data.borrow()
    }

    fn borrow_mut<'b>(self: &'b Rc<Self>) -> Self::BorrowMutGuard<'b> {
        self.data.borrow_mut()
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

mod playground {
    use std::rc::{Rc, Weak};

    use super::{mapper::Mapper, new::New, MappingEdge, Project, Projectable, Store};

    use super::EdgeTrait;

    struct MyStruct {
        f1: InnerStruct,
        f2: bool,
    }
    struct InnerStruct {
        i1: bool,
    }

    struct PMyStruct<P>
    where
        P: EdgeTrait<Data = MyStruct>,
    {
        pub f1: PInnerStruct<MappingEdge<P, MapperMyStateTof1>>,
        incoming_edge: Rc<P>,
    }

    impl<P> New for PMyStruct<P>
    where
        P: EdgeTrait<Data = MyStruct>,
    {
        type Arg = Rc<P>;
        fn new(arg: Self::Arg) -> Self {
            let f1 = New::new(Rc::new(MappingEdge::new(arg.clone(), MapperMyStateTof1)));
            Self {
                f1,
                incoming_edge: arg,
            }
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
        fn map<'s, 'd>(&'s self, input: &'d Self::In) -> &'d Self::Out {
            &input.f1
        }
        fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> &'d mut Self::Out {
            &mut input.f1
        }
    }
    struct PInnerStruct<P>
    where
        P: EdgeTrait<Data = InnerStruct>,
    {
        pub i1: Pbool<MappingEdge<P, MapperInnerStateToi1>>,
        incoming_edge: Rc<P>,
    }

    impl<N> New for PInnerStruct<N>
    where
        N: EdgeTrait<Data = InnerStruct>,
    {
        type Arg = Rc<N>;
        fn new(arg: Self::Arg) -> Self {
            let i1 = New::new(Rc::new(MappingEdge::new(arg.clone(), MapperInnerStateToi1)));
            Self {
                i1,
                incoming_edge: arg,
            }
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
        fn map<'s, 'd>(&'s self, input: &'d Self::In) -> &'d Self::Out {
            &input.i1
        }
        fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> &'d mut Self::Out {
            &mut input.i1
        }
    }
    pub struct Pbool<N>
    where
        N: EdgeTrait<Data = bool>,
    {
        node: Rc<N>,
    }

    impl<N> New for Pbool<N>
    where
        N: EdgeTrait<Data = bool>,
    {
        type Arg = Rc<N>;

        fn new(arg: Self::Arg) -> Self {
            Self { node: arg }
        }
    }

    impl<E> Projectable<bool> for E
    where
        E: EdgeTrait<Data = bool>,
    {
        type Projection = Pbool<E>;
    }
    fn hello() {
        let data = MyStruct {
            f1: InnerStruct { i1: false },
            f2: true,
        };
        let store = Store::new(data);
        let proj: PMyStruct<_> = store.project();
        let b = *proj.f1.i1.node.borrow();
        fn take(pr: Project<MyStruct, impl EdgeTrait<Data = MyStruct>>) {
            let b = *pr.f1.i1.node.borrow();
        }
    }
}
