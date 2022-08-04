use std::{marker::PhantomData, rc::Rc};

use x_bow_macros::Track;

use crate::trackable::{Trackable, TrackedPart};

#[derive(Track)]
#[x_bow(module_prefix = crate::__for_macro)]
struct Test<T> {
    value: T,
}
macro_rules! leaf_primitive {
    ($primitive:ty) => {
        impl<E> Trackable<E> for $primitive
        where
            E: crate::edge::EdgeTrait<Data = $primitive>,
        {
            type Tracked = super::TrackedLeaf<$primitive, E>;
        }
    };
}
leaf_primitive!(bool);
leaf_primitive!(char);
leaf_primitive!(f32);
leaf_primitive!(f64);
leaf_primitive!(i128);
leaf_primitive!(i16);
leaf_primitive!(i32);
leaf_primitive!(i64);
leaf_primitive!(i8);
leaf_primitive!(isize);
leaf_primitive!(u128);
leaf_primitive!(u16);
leaf_primitive!(u32);
leaf_primitive!(u64);
leaf_primitive!(u8);
leaf_primitive!(usize);

mod option {
    use x_bow_macros::Track;
    #[derive(Track)]
    #[x_bow(module_prefix = crate::__for_macro)]
    #[x_bow(remote_type = Option)]
    pub enum ImitateOption<T> {
        Some(T),
        None,
    }
}

// #[allow(non_snake_case)]
// pub struct POption<T, E>
// where
//     T: Trackable<Edge<E, MapperOption<T>, OptionalYes>>,
//     E: EdgeTrait<Data = Option<T>>,
// {
//     pub Some: TrackedPart<T, Edge<E, MapperOption<T>, OptionalYes>>,
//     incoming_edge: Rc<E>,
// }
// pub struct MapperOption<T>(PhantomData<T>);

// impl<T> Clone for MapperOption<T> {
//     fn clone(&self) -> Self {
//         Self(PhantomData)
//     }
// }
// impl<T> Mapper for MapperOption<T> {
//     type In = Option<T>;
//     type Out = T;
//     #[inline]
//     fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
//         input.as_ref()
//     }
//     #[inline]
//     fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
//         input.as_mut()
//     }
// }
// impl<T, E> Tracked for POption<T, E>
// where
//     E: EdgeTrait<Data = Option<T>>,
//     T: Trackable<Edge<E, MapperOption<T>, OptionalYes>>,
// {
//     type Edge = E;

//     fn new(edge: Rc<E>) -> Self {
//         Self {
//             Some: Tracked::new(Rc::new(Edge::new(edge.clone(), MapperOption(PhantomData)))),
//             incoming_edge: edge,
//         }
//     }
//     fn edge(&self) -> &Rc<Self::Edge> {
//         &self.incoming_edge
//     }
//     fn invalidate_here_down(&self) {
//         self.edge().invalidate_here();
//     }
// }
// impl<T, E> Trackable<E> for Option<T>
// where
//     T: Trackable<Edge<E, MapperOption<T>, OptionalYes>>,
//     E: EdgeTrait<Data = Option<T>>,
// {
//     type Tracked = POption<T, E>;
// }
