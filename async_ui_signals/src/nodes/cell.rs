use std::cell::RefCell;

use crate::{Visitable, sub::{SubManager, ParentSub}};

pub struct SignalCell<T> {
    value: RefCell<T>,
    sub: SubManager
}

impl<T> SignalCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
            sub: SubManager::new()
        }
    }
}

impl<'a, T, V> Visitable<V> for SignalCell<T>
where
    V: ?Sized + for<'x> FnMut(&'x T) + 'a
{
    fn visit<'v, 's>(&'s self, visitor: &'v mut V)
    where
        Self: 'v
    {
        visitor(&*self.value.borrow());
    }
    fn get_sub<'s>(&'s self) -> ParentSub<'s> {
        (&self.sub).into()
    }
}