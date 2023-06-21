mod borrow_mut;
mod for_each;
mod until_change;
use std::{cell::RefCell, fmt::Debug, task::Waker};

use smallvec::SmallVec;

use self::borrow_mut::ReactiveCellBorrowMut;

pub struct ReactiveCell<T> {
    inner: RefCell<Inner<T>>,
}

impl<T: Debug> Debug for ReactiveCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_tuple("ReactiveCell");
        match self.inner.try_borrow() {
            Ok(inside) => d.field(&inside.data).finish(),
            Err(_) => {
                // https://doc.rust-lang.org/src/core/fmt/mod.rs.html#2618
                struct BorrowedPlaceholder;
                impl Debug for BorrowedPlaceholder {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str("<borrowed>")
                    }
                }
                d.field(&BorrowedPlaceholder).finish()
            }
        }
    }
}

impl<T: Clone> Clone for ReactiveCell<T> {
    fn clone(&self) -> Self {
        let old_inner = self.inner.borrow();
        Self {
            inner: RefCell::new(Inner {
                data: old_inner.data.clone(),
                listeners: SmallVec::new(),
                version: 1,
            }),
        }
    }
}

impl<T: Default> Default for ReactiveCell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

struct Inner<T> {
    data: T,
    listeners: SmallVec<[Waker; 2]>,
    version: u64,
}


impl<T> ReactiveCell<T> {
    pub fn new(data: T) -> Self {
        let inner = RefCell::new(Inner {
            data,
            listeners: SmallVec::new(),
            version: 1,
        });
        Self { inner }
    }
    pub fn borrow(&'_ self) -> std::cell::Ref<'_, T> {
        std::cell::Ref::map(self.inner.borrow(), |r| &r.data)
    }
    pub fn borrow_mut(&'_ self) -> ReactiveCellBorrowMut<'_, T> {
        ReactiveCellBorrowMut {
            reference: self.inner.borrow_mut(),
        }
    }
}
