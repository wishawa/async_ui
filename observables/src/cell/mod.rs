mod borrow_mut;
use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    marker::PhantomData,
    task::Waker,
};

use smallvec::SmallVec;

use crate::{Listenable, Observable, ObservableBorrow, Version};

use self::borrow_mut::ReactiveCellBorrowMut;

pub struct ReactiveCell<T> {
    inner: RefCell<Inner<T>>,
}

struct Inner<T> {
    data: T,
    listeners: SmallVec<[Waker; 2]>,
    version: Version,
}

impl<T> ReactiveCell<T> {
    pub fn new(data: T) -> Self {
        let inner = RefCell::new(Inner {
            data,
            listeners: SmallVec::new(),
            version: Version::new(),
        });
        Self { inner }
    }
    pub fn borrow_mut<'b>(&'b self) -> ReactiveCellBorrowMut<'b, T> {
        ReactiveCellBorrowMut {
            reference: self.inner.borrow_mut(),
        }
    }
    pub fn set(&self, value: T) {
        *self.borrow_mut() = value;
    }
    pub fn as_observable<'b>(&'b self) -> ReactiveCellObservable<T, &'b Self> {
        ReactiveCellObservable {
            inner: self,
            _phantom: PhantomData,
        }
    }
}
pub struct ReactiveCellObservable<T, A: Borrow<ReactiveCell<T>>> {
    pub(crate) inner: A,
    pub(crate) _phantom: PhantomData<T>,
}
impl<T, A: Borrow<ReactiveCell<T>>> Listenable for ReactiveCellObservable<T, A> {
    fn add_waker(&self, waker: Waker) {
        self.inner.borrow().inner.borrow_mut().listeners.push(waker);
    }
    fn get_version(&self) -> Version {
        self.inner.borrow().inner.borrow().version
    }
}
impl<T, A: Borrow<ReactiveCell<T>>> Observable for ReactiveCellObservable<T, A> {
    type Data = T;
    fn borrow_observable<'b>(&'b self) -> ObservableBorrow<'b, T> {
        ObservableBorrow::RefCell(Ref::map(self.inner.borrow().inner.borrow(), |r| {
            r.data.borrow()
        }))
    }
}
