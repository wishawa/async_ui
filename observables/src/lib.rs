use std::{cell::Ref, ops::Deref, task::Waker};

use transformers::map::Map;
pub use version::Version;
mod impls;
mod next_change;
mod transformers;
mod version;
pub use next_change::NextChangeFuture;

pub mod cell;
#[cfg(feature = "futures-signals")]
pub mod futures_signals;

pub trait ObservableBase {
    fn add_waker(&self, waker: Waker);
    fn get_version(&self) -> Version;
}
pub enum ObservableBorrow<'b, T: ?Sized> {
    Borrow(&'b T),
    RefCell(Ref<'b, T>),
}

impl<'b, T: ?Sized> Deref for ObservableBorrow<'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            ObservableBorrow::Borrow(r) => *r,
            ObservableBorrow::RefCell(c) => c.deref(),
        }
    }
}
pub trait Observable<T: ?Sized>: ObservableBase {
    fn get_borrow<'b>(&'b self) -> ObservableBorrow<'b, T>;
}

pub trait ObservableExt<T: ?Sized>: Observable<T> {
    fn map<'w, O, M>(&'w self, mapper: M) -> Map<'w, Self, T, O, M>
    where
        M: Fn(&T) -> O,
        Self: Sized,
    {
        Map::new(self, mapper)
    }
    fn until_change<'i>(&'i self) -> NextChangeFuture<Self, &'i Self> {
        NextChangeFuture::new(self)
    }
}
impl<T: ?Sized, O: Observable<T> + ?Sized> ObservableExt<T> for O {}
