use std::cell::RefCell;

use crate::{
    listeners::Listener, node_up::NodeUpTrait, shared::Shared, trackable::Trackable, Store,
};

pub struct RootNodeUp<'u, T> {
    listener: Listener<'u>,
    data: RefCell<&'u mut T>,
}

impl<'u, T> NodeUpTrait for RootNodeUp<'u, T> {
    type Data = T;

    fn invalidate_upward_recursive(&self) {
        // no-op
    }
    fn up_borrow<'b>(&'b self) -> Option<std::cell::Ref<'b, Self::Data>> {
        Some(std::cell::Ref::map(self.data.borrow(), |d| &**d))
    }
    fn up_borrow_mut<'b>(&'b self) -> Option<std::cell::RefMut<'b, Self::Data>> {
        Some(std::cell::RefMut::map(self.data.borrow_mut(), |d| &mut **d))
    }
    fn get_listener<'b>(&'b self) -> &'b Listener {
        &self.listener
    }
}

pub struct UninitializedStore {
    shared: Shared,
}

pub fn create_store() -> UninitializedStore {
    UninitializedStore {
        shared: Shared::default(),
    }
}

impl UninitializedStore {
    pub fn initialize<'u, T: Trackable + 'u>(&'u self, data: &'u mut T) -> Store<'u, T, true> {
        let root = RootNodeUp {
            listener: Listener::new(&self.shared.wakers_list),
            data: RefCell::new(data),
        };
        let root = self.shared.allocator.alloc(root);
        T::new_node(&self.shared, root)
    }
}
// pub fn create_store<'u, T: Trackable + 'u>(shared: &'u Shared, data: &'u mut T) -> Store<'u, T, true> {
//     let root = RootNodeUp {
//         listener: Listener::new(&shared.wakers_list),
//         data: RefCell::new(data),
//     };
//     let root = shared.allocator.alloc(root);
//     T::new_node(shared, root)
// }
