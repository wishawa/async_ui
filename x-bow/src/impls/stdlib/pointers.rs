use crate::{
    mapper::Mapper,
    node_down::NodeDownTrait,
    node_up::{NodeUp, NodeUpTrait},
    track_api::Store,
    trackable::Trackable,
};
mod r#box {
    use std::marker::PhantomData;

    use crate::{is_guaranteed::IsGuaranteed, shared::Shared};

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
        up: &'u (dyn NodeUpTrait<Data = Box<T>> + 'u),
    }

    impl<'u, T, const G: bool> IsGuaranteed<G> for TrackedBox<'u, T, G> where T: Trackable + 'u {}

    impl<'u, T, const G: bool> NodeDownTrait<'u, Box<T>> for TrackedBox<'u, T, G>
    where
        T: Trackable + 'u,
    {
        fn invalidate_downward(&self) {
            self.node_up().invalidate_downward();
            self.inside.invalidate_downward();
        }

        fn node_up(&self) -> &'u (dyn NodeUpTrait<Data = Box<T>> + 'u) {
            self.up
        }
    }

    impl<T: Trackable> Trackable for Box<T> {
        type NodeDown<'u, const G: bool> = TrackedBox<'u, T, G> where Self : 'u;

        fn new_node<'u, Up: NodeUpTrait<Data = Self> + 'u, const G: bool>(
            shared: &'u Shared,
            up_node: &'u Up,
        ) -> Self::NodeDown<'u, G>
        where
            Self: 'u,
        {
            TrackedBox {
                inside: T::new_node(
                    shared,
                    shared.allocator.alloc(NodeUp::new(
                        shared,
                        up_node.clone(),
                        MapperBox(PhantomData),
                    )),
                ),
                up: up_node,
            }
        }
    }
}
