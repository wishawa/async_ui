use std::{
    future::Future,
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

pub trait ObservableBase {
    fn add_waker(&self, waker: Waker);
    fn get_version(&self) -> Version;
}
pub trait Observable: ObservableBase {
    type Data;
    fn visit<R, F: FnOnce(&Self::Data) -> R>(&self, func: F) -> R;
}

pub trait ObservableExt: Observable {
    fn map<O, M>(self, mapper: M) -> Map<Self, O, M>
    where
        M: Fn(&Self::Data) -> O,
        Self: Sized,
    {
        Map::new(self, mapper)
    }
    fn until_change<'i>(&'i self) -> NextChangeFuture<'i, Self> {
        NextChangeFuture {
            inner: self,
            start_version: Version::new_null(),
        }
    }
}
impl<T: Observable> ObservableExt for T {}

pub struct NextChangeFuture<'i, I>
where
    I: ObservableBase,
    I: ?Sized,
{
    inner: &'i I,
    start_version: Version,
}

impl<'i, I> Future for NextChangeFuture<'i, I>
where
    I: ObservableBase,
    I: ?Sized,
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
