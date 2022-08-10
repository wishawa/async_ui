use std::{marker::PhantomData, task::Waker};

use crate::{Observable, ObservableBase, Version};

pub struct Map<W, I, O, M>
where
    W: Observable<I>,
    M: Fn(&I) -> O,
{
    wrapped: W,
    mapper: M,
    _phantom: PhantomData<(I, O)>,
}

impl<W, I, O, M> Map<W, I, O, M>
where
    W: Observable<I>,
    M: Fn(&I) -> O,
{
    pub(crate) fn new(wrapped: W, mapper: M) -> Self {
        Self {
            wrapped,
            mapper,
            _phantom: PhantomData,
        }
    }
}

impl<W, I, O, M> Observable<O> for Map<W, I, O, M>
where
    W: Observable<I>,
    M: Fn(&I) -> O,
{
    fn visit<R, F: FnOnce(&O) -> R>(&self, func: F) -> R {
        self.wrapped.visit(|input| {
            let output = (self.mapper)(input);
            func(&output)
        })
    }
}

impl<W, I, O, M> ObservableBase<O> for Map<W, I, O, M>
where
    W: Observable<I>,
    M: Fn(&I) -> O,
{
    fn add_waker(&self, waker: Waker) {
        self.wrapped.add_waker(waker)
    }

    fn get_version(&self) -> Version {
        self.wrapped.get_version()
    }
}
