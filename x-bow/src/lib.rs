#![feature(generic_associated_types)]
mod borrow_output;
mod deref_optional;
mod edge;
mod impls;
mod listeners;
mod mapper;
mod optional;
mod store;
mod trackable;
mod tracked;
pub use x_bow_macros::Track;

#[doc(hidden)]
pub mod __private_macro_only {
    pub use super::edge::{Edge, EdgeTrait};
    pub use super::impls::TrackedLeaf;
    pub use super::mapper::Mapper;
    pub use super::optional::{IsOptional, OptionalNo, OptionalYes};
    pub use super::trackable::{HandlePart, Trackable};
    pub use super::tracked::Tracked;
}
pub use store::{Handle, Store};
pub use tracked::{TrackedExt, TrackedExtGuaranteed};

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}

// mod playground {
//     use std::rc::Rc;

//     use crate::edge::{Edge, EdgeTrait};
//     use crate::impls::TrackedLeaf;
//     use crate::mapper::Mapper;
//     use crate::store::{Handle, Store};
//     use crate::trackable::{HandlePart, Trackable};
//     use crate::tracked::Tracked;

//     struct MyStruct {
//         f1: InnerStruct,
//         f2: bool,
//     }
//     struct InnerStruct {
//         i1: bool,
//         i2: Option<bool>,
//     }

//     struct PMyStruct<P>
//     where
//         P: EdgeTrait<Data = MyStruct>,
//     {
//         // pub f1: PInnerStruct<Edge<P, MapperMyStateTof1, P::Optional>>,
//         pub f1: HandlePart<InnerStruct, Edge<P, MapperMyStateTof1, P::Optional>>,
//         incoming_edge: Rc<P>,
//     }

//     impl<P> Tracked for PMyStruct<P>
//     where
//         P: EdgeTrait<Data = MyStruct>,
//     {
//         type Edge = P;
//         fn new(edge: Rc<P>) -> Self {
//             let f1 = Tracked::new(Rc::new(Edge::new(edge.clone(), MapperMyStateTof1)));
//             Self {
//                 f1,
//                 incoming_edge: edge,
//             }
//         }
//         fn edge(&self) -> &Rc<Self::Edge> {
//             &self.incoming_edge
//         }
//         fn invalidate_down_outside(&self) {
//             self.edge().invalidate_outside_here();
//             self.f1.invalidate_down_outside();
//         }
//     }
//     impl<E> Trackable<E> for MyStruct
//     where
//         E: EdgeTrait<Data = Self>,
//     {
//         type Tracked = PMyStruct<E>;
//     }

//     #[derive(Clone)]
//     struct MapperMyStateTof1;
//     impl Mapper for MapperMyStateTof1 {
//         type In = MyStruct;
//         type Out = InnerStruct;
//         fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
//             Some(&input.f1)
//         }
//         fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
//             Some(&mut input.f1)
//         }
//     }
//     struct PInnerStruct<P>
//     where
//         P: EdgeTrait<Data = InnerStruct>,
//     {
//         pub i1: TrackedLeaf<bool, Edge<P, MapperInnerStateToi1, P::Optional>>,
//         // pub i1: PLeaf<bool, Edge<P, MapperInnerStateToi1, P::Optional>>,
//         pub i2: TrackedLeaf<Option<bool>, Edge<P, MapperInnerStateToi2, P::Optional>>,
//         // pub i2: POption<bool, Edge<P, MapperInnerStateToi2, P::Optional>>,
//         pub i22: TrackedLeaf<Option<bool>, Edge<P, MapperInnerStateToi2, P::Optional>>,
//         incoming_edge: Rc<P>,
//     }

//     impl<N> Tracked for PInnerStruct<N>
//     where
//         N: EdgeTrait<Data = InnerStruct>,
//     {
//         type Edge = N;
//         fn new(edge: Rc<N>) -> Self {
//             let i1 = Tracked::new(Rc::new(Edge::new(edge.clone(), MapperInnerStateToi1)));
//             let i2 = Tracked::new(Rc::new(Edge::new(edge.clone(), MapperInnerStateToi2)));
//             let i22 = Tracked::new(Rc::new(Edge::new(edge.clone(), MapperInnerStateToi2)));
//             Self {
//                 i1,
//                 i2,
//                 i22,
//                 incoming_edge: edge,
//             }
//         }
//         fn edge(&self) -> &Rc<Self::Edge> {
//             &self.incoming_edge
//         }

//         fn invalidate_down_outside(&self) {
//             self.edge().invalidate_outside_here();
//             self.i1.invalidate_down_outside();
//         }
//     }
//     impl<E> Trackable<E> for InnerStruct
//     where
//         E: EdgeTrait<Data = Self>,
//     {
//         type Tracked = PInnerStruct<E>;
//     }

//     #[derive(Clone)]
//     struct MapperInnerStateToi1;
//     impl Mapper for MapperInnerStateToi1 {
//         type In = InnerStruct;
//         type Out = bool;
//         fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
//             Some(&input.i1)
//         }
//         fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
//             Some(&mut input.i1)
//         }
//     }
//     #[derive(Clone)]
//     struct MapperInnerStateToi2;
//     impl Mapper for MapperInnerStateToi2 {
//         type In = InnerStruct;
//         type Out = Option<bool>;
//         fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
//             Some(&input.i2)
//         }
//         fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
//             Some(&mut input.i2)
//         }
//     }

//     // impl<E> Projectable<E> for bool
//     // where
//     //     E: EdgeTrait<Data = bool>,
//     // {
//     //     type Projection = ProjectedLeaf<bool, E>;
//     // }

//     fn hello() {
//         use crate::tracked::{TrackedExt, TrackedExtGuaranteed};
//         let data = MyStruct {
//             f1: InnerStruct {
//                 i2: Some(true),
//                 i1: false,
//             },
//             f2: true,
//         };
//         let proj = Store::new(data);
//         let b = *proj.f1.i1.borrow_opt().unwrap();
//         let c = &*proj.f1.borrow_opt().unwrap();
//         let b = &*proj.f1.borrow();
//         let b = &*proj.f1.i2.borrow();
//         // let b = &proj.f1.i2.Some;
//         let b = &*proj.f1.i22.borrow();

//         // let b = *proj.f1.i2.Some.borrow_opt().unwrap();
//         take(&proj);
//         take2(&proj.f1);
//         fn take(proj: &Handle<MyStruct>) {
//             let b = *proj.f1.i1.borrow_opt().unwrap();
//             // take2(&proj.f1);
//             // take3(&proj.f1);
//         }
//         fn take2(proj: &HandlePart<InnerStruct, impl EdgeTrait<Data = InnerStruct>>) {
//             let a = proj.i2.borrow_opt();
//         }
//     }
// }
