use std::{
    cell::{Ref, RefMut},
    rc::Rc,
};

use crate::{listeners::Listener, mapper::Mapper};

pub trait NodeUpTrait {
    type Data;
    /// Invalidate every ancestor.
    fn invalidate_up(&self);
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
    parent: Rc<dyn NodeUpTrait<Data = M::In> + 'u>,
    mapper: M,
    listener: Listener,
}

impl<'u, M: Mapper> NodeUp<'u, M> {
    pub fn new(parent: Rc<dyn NodeUpTrait<Data = M::In> + 'u>, mapper: M) -> Self {
        Self {
            parent,
            mapper,
            listener: Listener::new(),
        }
    }
}

impl<'u, M: Mapper> NodeUpTrait for NodeUp<'u, M> {
    type Data = M::Out;

    fn invalidate_up(&self) {
        self.parent.get_listener().down().increment_version();
        self.parent.invalidate_up();
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
