use crate::{
    mapper::Mapper,
    node_down::NodeDownTrait,
    node_up::{NodeUp, NodeUpTrait},
    track_api::Store,
    trackable::Trackable,
};
mod r#box {
    use std::{marker::PhantomData, rc::Rc};

    use crate::is_guaranteed::IsGuaranteed;

    use super::*;

    struct MapperBox<T>(PhantomData<T>);
    impl<T> Mapper for MapperBox<T> {
        type In = Box<T>;
        type Out = T;

        fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
            Some(&*input)
        }

        fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
            Some(&mut *input)
        }
    }

    pub struct TrackedBox<'u, T, const G: bool>
    where
        T: Trackable + 'u,
    {
        inside: Store<'u, T, G>,
        up: Rc<dyn NodeUpTrait<Data = Box<T>> + 'u>,
    }

    impl<'u, T, const G: bool> Clone for TrackedBox<'u, T, G>
    where
        T: Trackable + 'u,
    {
        fn clone(&self) -> Self {
            Self {
                inside: self.inside.clone(),
                up: self.up.clone(),
            }
        }
    }

    impl<'u, T, const G: bool> IsGuaranteed<G> for TrackedBox<'u, T, G> where T: Trackable + 'u {}

    impl<'u, T, const G: bool> NodeDownTrait<'u, Box<T>> for TrackedBox<'u, T, G>
    where
        T: Trackable + 'u,
    {
        fn invalidate_down(&self) {
            self.node_up().get_listener().invalidate_down();
            self.inside.invalidate_down();
        }

        fn node_up(&self) -> &Rc<dyn NodeUpTrait<Data = Box<T>> + 'u> {
            &self.up
        }
    }

    impl<T: Trackable> Trackable for Box<T> {
        type NodeDown<'u, const G: bool> = TrackedBox<'u, T, G> where Self : 'u;

        fn new_node<'u, const G: bool>(
            up_node: std::rc::Rc<dyn NodeUpTrait<Data = Box<T>> + 'u>,
        ) -> Self::NodeDown<'u, G>
        where
            Self: 'u,
        {
            TrackedBox {
                inside: T::new_node(Rc::new(NodeUp::new(
                    up_node.clone(),
                    MapperBox(PhantomData),
                ))),
                up: up_node,
            }
        }
    }
}
