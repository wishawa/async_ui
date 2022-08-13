use std::{
    cell::{Ref, RefCell},
    task::Waker,
};

use crate::{Observable, ObservableBase, ObservableBorrow, Version};

pub struct Map<'i, I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    wrapped: &'i I,
    mapper: M,
    last_value: RefCell<Option<O>>,
}

impl<'i, I, O, M> Map<'i, I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    pub(crate) fn new(wrapped: &'i I, mapper: M) -> Self {
        Self {
            wrapped,
            mapper,
            last_value: Default::default(),
        }
    }
}

impl<'i, I, O, M> Observable for Map<'i, I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    type Data = O;
    fn get_borrow<'b>(&'b self) -> ObservableBorrow<'b, Self::Data> {
        let input = self.wrapped.get_borrow();
        let mapped = (self.mapper)(&*input);
        {
            *self.last_value.borrow_mut() = Some(mapped);
        }
        ObservableBorrow::RefCell(Ref::map(self.last_value.borrow(), |v| v.as_ref().unwrap()))
    }
}

impl<'i, I, O, M> ObservableBase for Map<'i, I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    fn add_waker(&self, waker: Waker) {
        self.wrapped.add_waker(waker)
    }

    fn get_version(&self) -> Version {
        self.wrapped.get_version()
    }
}
