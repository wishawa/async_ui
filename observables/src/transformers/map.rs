use std::{cell::RefCell, marker::PhantomData, task::Waker};

use crate::{Listenable, ObservableAs, ObservableAsExt, ObservableBase, Version};

pub struct Map<W, I, O, M>
where
    W: ObservableAs<I>,
    M: Fn(&I) -> O,
    I: ?Sized,
{
    wrapped: W,
    mapper: M,
    last_value: RefCell<Option<O>>,
    _phantom: PhantomData<I>,
}

impl<W, I, O, M> Map<W, I, O, M>
where
    W: ObservableAs<I>,
    M: Fn(&I) -> O,
    I: ?Sized,
{
    pub(crate) fn new(wrapped: W, mapper: M) -> Self {
        Self {
            wrapped,
            mapper,
            last_value: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<W, I, O, M> ObservableBase for Map<W, I, O, M>
where
    W: ObservableAs<I>,
    M: Fn(&I) -> O,
    I: ?Sized,
{
    type Data = O;

    fn visit_base<'b, F: FnOnce(&Self::Data) -> U, U>(&'b self, f: F) -> U {
        self.wrapped.visit(|w| {
            {
                *self.last_value.borrow_mut() = Some((self.mapper)(w));
            }
            f(self.last_value.borrow().as_ref().unwrap())
        })
    }
}

impl<W, I, O, M> Listenable for Map<W, I, O, M>
where
    W: ObservableAs<I>,
    M: Fn(&I) -> O,
    I: ?Sized,
{
    fn add_waker(&self, waker: Waker) {
        self.wrapped.add_waker(waker)
    }

    fn get_version(&self) -> Version {
        self.wrapped.get_version()
    }
}
