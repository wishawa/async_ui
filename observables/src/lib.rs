use std::{
    cell::Ref,
    ops::Deref,
    rc::Rc,
    sync::{Arc, MutexGuard, RwLockReadGuard},
    task::Waker,
};

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
    Mutex(MutexGuard<'b, T>),
    RwLock(RwLockReadGuard<'b, T>),
    OtherBoxed(Box<dyn Deref<Target = T> + 'b>),
    OtherRc(Rc<dyn Deref<Target = T> + 'b>),
    OtherArc(Arc<dyn Deref<Target = T> + 'b>),
}

impl<'b, T: ?Sized> Deref for ObservableBorrow<'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            ObservableBorrow::Borrow(x) => x.deref(),
            ObservableBorrow::RefCell(x) => x.deref(),
            ObservableBorrow::Mutex(x) => x.deref(),
            ObservableBorrow::RwLock(x) => x.deref(),
            ObservableBorrow::OtherBoxed(x) => x.deref(),
            ObservableBorrow::OtherRc(x) => x.deref(),
            ObservableBorrow::OtherArc(x) => x.deref(),
        }
    }
}
pub trait Observable<T: ?Sized>: ObservableBase {
    fn borrow_observable<'b>(&'b self) -> ObservableBorrow<'b, T>;
}

pub trait ObservableExt<T: ?Sized>: Observable<T> {
    fn map<O, M>(self, mapper: M) -> Map<Self, T, O, M>
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
