mod borrow_mut;
use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    task::Waker,
};

use crate::{Observable, ObservableBase, ObservableBorrow, Version};

use self::borrow_mut::ObservableCellBorrowMut;

pub struct ObservableCell<T> {
    inner: RefCell<ObserbableCellInner<T>>,
}

struct ObserbableCellInner<T> {
    data: T,
    listeners: Vec<Waker>,
    version: Version,
}

impl<T> ObservableCell<T> {
    pub fn new(data: T) -> Self {
        let inner = RefCell::new(ObserbableCellInner {
            data,
            listeners: Vec::new(),
            version: Version::new(),
        });
        Self { inner }
    }
    pub fn borrow_mut<'b>(&'b self) -> ObservableCellBorrowMut<'b, T> {
        ObservableCellBorrowMut {
            reference: self.inner.borrow_mut(),
        }
    }
    pub fn as_observable<'b>(&'b self) -> ObservableCellObservable<'b, T> {
        ObservableCellObservable { inner: self }
    }
}
pub struct ObservableCellObservable<'c, T> {
    inner: &'c ObservableCell<T>,
}
impl<'a, T> ObservableBase for ObservableCellObservable<'a, T> {
    fn add_waker(&self, waker: Waker) {
        self.inner.inner.borrow_mut().listeners.push(waker);
    }
    fn get_version(&self) -> Version {
        self.inner.inner.borrow().version
    }
}
impl<'a, T, Z: ?Sized> Observable<Z> for ObservableCellObservable<'a, T>
where
    T: Borrow<Z>,
{
    fn borrow_observable<'b>(&'b self) -> ObservableBorrow<'b, Z> {
        ObservableBorrow::RefCell(Ref::map(self.inner.inner.borrow(), |r| r.data.borrow()))
    }
}
