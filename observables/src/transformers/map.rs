use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    marker::PhantomData,
    task::Waker,
};

use crate::{Observable, ObservableBase, ObservableBorrow, Version};

pub struct Map<'w, W, I, O, M>
where
    W: Observable<I>,
    M: Fn(&I) -> O,
    I: ?Sized,
{
    wrapped: &'w W,
    mapper: M,
    last_value: RefCell<Option<O>>,
    _phantom: PhantomData<I>,
}

impl<'w, W, I, O, M> Map<'w, W, I, O, M>
where
    W: Observable<I>,
    M: Fn(&I) -> O,
    I: ?Sized,
{
    pub(crate) fn new(wrapped: &'w W, mapper: M) -> Self {
        Self {
            wrapped,
            mapper,
            last_value: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<'w, W, I, O, M, Z> Observable<Z> for Map<'w, W, I, O, M>
where
    W: Observable<I>,
    M: Fn(&I) -> O,
    O: Borrow<Z>,
    I: ?Sized,
    Z: ?Sized,
{
    fn get_borrow<'b>(&'b self) -> ObservableBorrow<'b, Z> {
        let input = self.wrapped.get_borrow();
        let mapped = (self.mapper)(&*input);
        {
            *self.last_value.borrow_mut() = Some(mapped);
        }
        ObservableBorrow::RefCell(Ref::map(self.last_value.borrow(), |v| {
            v.as_ref().unwrap().borrow()
        }))
    }
}

impl<'w, W, I, O, M> ObservableBase for Map<'w, W, I, O, M>
where
    W: Observable<I>,
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
