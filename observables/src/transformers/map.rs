use std::task::Waker;

use pin_project_lite::pin_project;

use crate::{Observable, ObservableBase};

pin_project! {
    pub struct Map<I, O, M>
    where
        I: Observable,
        M: Fn(&I::Data) -> O
    {
        #[pin]
        wrapped: I,
        mapper: M
    }
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
    fn add_waker(self: std::pin::Pin<&Self>, waker: Waker) {
        self.project_ref().wrapped.add_waker(waker)
    }

    fn get_version(self: std::pin::Pin<&Self>) -> u64 {
        self.project_ref().wrapped.get_version()
    }
}
