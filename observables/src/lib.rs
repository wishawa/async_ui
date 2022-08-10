use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use transformers::map::Map;
pub use version::Version;
mod transformers;
mod version;

pub mod cell;
#[cfg(feature = "futures-signals")]
pub mod futures_signals;

pub trait ObservableBase<V> {
    fn add_waker(&self, waker: Waker);
    fn get_version(&self) -> Version;
}
pub trait Observable<V>: ObservableBase<V> {
    fn visit<R, F: FnOnce(&V) -> R>(&self, func: F) -> R;
}

pub trait ObservableExt<V>: Observable<V> {
    fn map<O, M>(self, mapper: M) -> Map<Self, V, O, M>
    where
        M: Fn(&V) -> O,
        Self: Sized,
    {
        Map::new(self, mapper)
    }
    fn until_change<'s>(&'s self) -> NextChangeFuture<'s, Self, V> {
        NextChangeFuture {
            inner: self,
            start_version: Version::new_null(),
            _phantom: PhantomData,
        }
    }
}
impl<V, T: Observable<V>> ObservableExt<V> for T {}

pub struct NextChangeFuture<'w, W, V>
where
    W: ObservableBase<V>,
    W: ?Sized,
{
    inner: &'w W,
    start_version: Version,
    _phantom: PhantomData<Box<V>>,
}

impl<'w, W, V> Future for NextChangeFuture<'w, W, V>
where
    W: ObservableBase<V>,
    W: ?Sized,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.start_version.is_null() {
            this.start_version = this.inner.get_version();
            this.inner.add_waker(cx.waker().to_owned());
        }
        if this.inner.get_version() > this.start_version {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<T> ObservableBase<T> for (T,) {
    fn add_waker(&self, waker: Waker) {
        todo!()
    }

    fn get_version(&self) -> Version {
        todo!()
    }
}
