use std::task::Waker;

use crate::{Observable, ObservableBase, Version};

pub struct Map<I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    wrapped: I,
    mapper: M,
}

impl<I, O, M> Map<I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    pub(crate) fn new(wrapped: I, mapper: M) -> Self {
        Self { wrapped, mapper }
    }
}

impl<I, O, M> Observable for Map<I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    type Data = O;

    fn visit<R, F: FnOnce(&Self::Data) -> R>(&self, func: F) -> R {
        self.wrapped.visit(|input| {
            let output = (self.mapper)(input);
            func(&output)
        })
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
