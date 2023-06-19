use std::{cell::RefCell, rc::Rc};

use crate::{__private_macro_only::NodeUpTrait, listeners::Listener, trackable::Trackable, Store};

pub struct RootNodeUp<T> {
    listener: Listener,
    data: RefCell<T>,
}

impl<T> NodeUpTrait for RootNodeUp<T> {
    type Data = T;

    fn invalidate_up(&self) {
        // no-op
    }
    fn up_borrow<'b>(&'b self) -> Option<std::cell::Ref<'b, Self::Data>> {
        Some(self.data.borrow())
    }
    fn up_borrow_mut<'b>(&'b self) -> Option<std::cell::RefMut<'b, Self::Data>> {
        Some(self.data.borrow_mut())
    }
    fn get_listener<'b>(&'b self) -> &'b Listener {
        &self.listener
    }
}

pub fn create_store<'a, T: Trackable + 'a>(data: T) -> Store<'a, T, true> {
    T::new_node(Rc::new(RootNodeUp {
        listener: Listener::new(),
        data: RefCell::new(data),
    }))
}
