#![feature(generic_associated_types)]
mod borrow_output;
mod deref_optional;
mod edge;
mod impls;
mod in_enum;
mod listeners;
mod mapper;
mod projectable;
mod projection;
mod store;

mod __for_macro {
    pub use super::edge::{Edge, EdgeTrait};
    pub use super::impls::ProjectedLeaf;
    pub use super::mapper::Mapper;
    pub use super::projectable::ProjectedPart;
    pub use super::projection::Projection;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}

mod playground {
    use std::rc::Rc;

    use crate::edge::{Edge, EdgeTrait};
    use crate::impls::ProjectedLeaf;
    use crate::mapper::Mapper;
    use crate::projectable::{Projectable, ProjectedPart};
    use crate::projection::Projection;
    use crate::store::{Projected, Store};

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
        // pub f1: PInnerStruct<Edge<P, MapperMyStateTof1, P::InEnum>>,
        pub f1: ProjectedPart<InnerStruct, Edge<P, MapperMyStateTof1, P::InEnum>>,
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
    impl<E> Projectable<E> for MyStruct
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
        pub i1: ProjectedPart<bool, Edge<P, MapperInnerStateToi1, P::InEnum>>,
        // pub i1: PLeaf<bool, Edge<P, MapperInnerStateToi1, P::InEnum>>,
        pub i2: ProjectedPart<Option<bool>, Edge<P, MapperInnerStateToi2, P::InEnum>>,
        // pub i2: POption<bool, Edge<P, MapperInnerStateToi2, P::InEnum>>,
        pub i22: ProjectedLeaf<Option<bool>, Edge<P, MapperInnerStateToi2, P::InEnum>>,
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
            let i22 = Projection::new(Rc::new(Edge::new(edge.clone(), MapperInnerStateToi2)));
            Self {
                i1,
                i2,
                i22,
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
    impl<E> Projectable<E> for InnerStruct
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

    impl<E> Projectable<E> for bool
    where
        E: EdgeTrait<Data = bool>,
    {
        type Projection = ProjectedLeaf<bool, E>;
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
        let b = &*proj.f1.i22.borrow();

        // let b = *proj.f1.i2.Some.borrow_opt().unwrap();
        take(&proj);
        take2(&proj.f1);
        fn take(proj: &Projected<MyStruct>) {
            let b = *proj.f1.i1.borrow_opt().unwrap();
            // take2(&proj.f1);
            // take3(&proj.f1);
        }
        fn take2(proj: &ProjectedPart<InnerStruct, impl EdgeTrait<Data = InnerStruct>>) {
            let a = proj.i2.borrow_opt();
        }
    }
}
