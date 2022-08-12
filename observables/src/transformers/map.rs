use std::{
    cell::{Ref, RefCell},
    task::Waker,
};

use crate::{Observable, ObservableBase, ObservableBorrowed, Version};

pub struct Map<I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    wrapped: I,
    mapper: M,
    last_value: RefCell<Option<O>>,
}

impl<I, O, M> Map<I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    pub(crate) fn new(wrapped: I, mapper: M) -> Self {
        Self {
            wrapped,
            mapper,
            last_value: Default::default(),
        }
    }
}

impl<I, O, M> Observable for Map<I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    type Data = O;
    fn obs_borrow<'b>(&'b self) -> ObservableBorrowed<'b, Self::Data> {
        let input = self.wrapped.obs_borrow();
        let mapped = (self.mapper)(&*input);
        {
            *self.last_value.borrow_mut() = Some(mapped);
        }
        ObservableBorrowed::RefCell(Ref::map(self.last_value.borrow(), |v| v.as_ref().unwrap()))
    }
}

impl<I, O, M> ObservableBase for Map<I, O, M>
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
