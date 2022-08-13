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
}
impl<T> ObservableBase for ObservableCell<T> {
    fn add_waker(&self, waker: Waker) {
        self.inner.borrow_mut().listeners.push(waker);
    }
    fn get_version(&self) -> Version {
        self.inner.borrow().version
    }
}
impl<T, Z: ?Sized> Observable<Z> for ObservableCell<T>
where
    T: Borrow<Z>,
{
    fn get_borrow<'b>(&'b self) -> ObservableBorrow<'b, Z> {
        ObservableBorrow::RefCell(Ref::map(self.inner.borrow(), |r| r.data.borrow()))
    }
}
