use std::{
    borrow::Borrow,
    cell::Ref,
    ops::Deref,
    // sync::{Arc, MutexGuard, RwLockReadGuard},
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

pub trait Listenable {
    fn add_waker(&self, waker: Waker);
    fn get_version(&self) -> Version;
}
pub enum ObservableBorrow<'b, T: ?Sized> {
    Borrow(&'b T),
    RefCell(Ref<'b, T>),
    // Mutex(MutexGuard<'b, T>),
    // RwLock(RwLockReadGuard<'b, T>),
    // OtherBoxed(Box<dyn Deref<Target = T> + 'b>),
    // OtherRc(Rc<dyn Deref<Target = T> + 'b>),
    // OtherArc(Arc<dyn Deref<Target = T> + 'b>),
}

impl<'b, T: ?Sized> Deref for ObservableBorrow<'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            ObservableBorrow::Borrow(x) => x.deref(),
            ObservableBorrow::RefCell(x) => x.deref(),
            // ObservableBorrow::Mutex(x) => x.deref(),
            // ObservableBorrow::RwLock(x) => x.deref(),
            // ObservableBorrow::OtherBoxed(x) => x.deref(),
            // ObservableBorrow::OtherRc(x) => x.deref(),
            // ObservableBorrow::OtherArc(x) => x.deref(),
        }
    }
}
impl<'b, T: ?Sized> ObservableBorrow<'b, T> {
    fn map_to<U: ?Sized, M: Fn(&T) -> &U>(self, mapper: M) -> ObservableBorrow<'b, U> {
        match self {
            ObservableBorrow::Borrow(x) => ObservableBorrow::Borrow(mapper(x)),
            ObservableBorrow::RefCell(x) => ObservableBorrow::RefCell(Ref::map(x, mapper)),
        }
    }
}
pub trait Observable: Listenable {
    type Data: ?Sized;
    fn borrow_observable<'b>(&'b self) -> ObservableBorrow<'b, Self::Data>;
}

pub trait ObservableAs<Z: ?Sized>: Listenable {
    fn borrow_observable_as<'b>(&'b self) -> ObservableBorrow<'b, Z>;
}
pub trait ObservableAsExt<Z: ?Sized>: ObservableAs<Z> {
    fn map<O, M>(self, mapper: M) -> Map<Self, Z, O, M>
    where
        M: Fn(&Z) -> O,
        Self: Sized,
    {
        Map::new(self, mapper)
    }
    fn until_change<'i>(&'i self) -> NextChangeFuture<Self, &'i Self> {
        NextChangeFuture::new(self)
    }
}
impl<Z, O> ObservableAs<Z> for O
where
    Z: ?Sized,
    O: Observable + ?Sized,
    O::Data: Borrow<Z>,
{
    fn borrow_observable_as<'b>(&'b self) -> ObservableBorrow<'b, Z> {
        self.borrow_observable().map_to(Borrow::borrow)
    }
}

impl<Z, O> ObservableAsExt<Z> for O
where
    Z: ?Sized,
    O: ObservableAs<Z> + ?Sized,
{
}
