use std::cell::{Ref, RefMut};

use crate::{listeners::Listener, mapper::Mapper, shared::Shared};

pub trait NodeUpTrait {
    type Data;
    /// Invalidate every ancestor.
    fn invalidate_upward_recursive(&self);

    /// Invalidate just this node for "here" change listeners.
    fn invalidate_here(&self) {
        let listener = self.get_listener();
        listener.up_here_down[1].increment_version(&mut *listener.full_list.borrow_mut());
    }

    /// Invalidate just this node for "outside" change listeners.
    fn invalidate_downward(&self) {
        let listener = self.get_listener();
        listener.up_here_down[0].increment_version(&mut *listener.full_list.borrow_mut());
    }

    /// Borrow the data here immutably.
    fn up_borrow<'b>(&'b self) -> Option<Ref<'b, Self::Data>>;
    /// Borrow the data here mutably.
    /// Caller is responsible for performing the appropriate invalidation markings.
    fn up_borrow_mut<'b>(&'b self) -> Option<RefMut<'b, Self::Data>>;
    /// Get the [Listener] associated with this node.
    /// Can be used to subscribe or fire change events.
    fn get_listener<'b>(&'b self) -> &'b Listener;
}

pub struct NodeUp<'u, M: Mapper> {
    parent: &'u (dyn NodeUpTrait<Data = M::In> + 'u),
    mapper: M,
    listener: Listener<'u>,
}

impl<'u, M: Mapper> NodeUp<'u, M> {
    pub fn new(
        shared: &'u Shared,
        parent: &'u (dyn NodeUpTrait<Data = M::In> + 'u),
        mapper: M,
    ) -> Self {
        Self {
            parent,
            mapper,
            listener: Listener::new(&shared.wakers_list),
        }
    }
}

impl<'u, M: Mapper> NodeUpTrait for NodeUp<'u, M> {
    type Data = M::Out;

    fn invalidate_upward_recursive(&self) {
        self.parent.invalidate_upward_recursive();
        let listener = self.parent.get_listener();
        listener.up_here_down[2].increment_version(&mut *listener.full_list.borrow_mut());
    }

    fn up_borrow<'b>(&'b self) -> Option<Ref<'b, Self::Data>> {
        self.parent
            .up_borrow()
            .and_then(|r| Ref::filter_map(r, |d| self.mapper.map(d)).ok())
    }

    fn up_borrow_mut<'b>(&'b self) -> Option<RefMut<'b, Self::Data>> {
        self.parent
            .up_borrow_mut()
            .and_then(|r| RefMut::filter_map(r, |d| self.mapper.map_mut(d)).ok())
    }

    fn get_listener<'b>(&'b self) -> &'b Listener {
        &self.listener
    }
}
