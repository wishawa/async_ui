use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use borrowed::Borrowed;
use transformers::map::Map;
pub use version::Version;
mod impls;
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
    type Data: ?Sized;
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
    fn until_change<'i>(&'i self) -> NextChangeFuture<Borrowed<'i, Self>, Self> {
        NextChangeFuture::new(Borrowed(self))
    }
}
impl<T: Observable> ObservableExt for T {}
mod borrowed {
    pub struct Borrowed<'t, T: ?Sized>(pub &'t T);

    impl<'t, T: ?Sized> AsRef<T> for Borrowed<'t, T> {
        fn as_ref(&self) -> &T {
            &self.0
        }
    }
}

pub struct NextChangeFuture<A, I>
where
    A: AsRef<I> + Unpin,
    I: ObservableBase,
    I: ?Sized,
{
    inner: A,
    start_version: Version,
    _phantom: PhantomData<Box<I>>,
}

impl<A, I> NextChangeFuture<A, I>
where
    A: AsRef<I> + Unpin,
    I: ObservableBase,
    I: ?Sized,
{
    pub fn new(observable: A) -> Self {
        Self {
            inner: observable,
            start_version: Version::new_null(),
            _phantom: PhantomData,
        }
    }
}

impl<A, I> Future for NextChangeFuture<A, I>
where
    A: AsRef<I> + Unpin,
    I: ObservableBase,
    I: ?Sized,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let inner = this.inner.as_ref();
        if this.start_version.is_null() {
            this.start_version = inner.get_version();
            inner.add_waker(cx.waker().to_owned());
        }
        if inner.get_version() > this.start_version {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
