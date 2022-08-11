use pin_project_lite::pin_project;
use std::{
    borrow::Borrow,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, Waker},
};

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
    fn until_change<'i>(&'i self) -> NextChangeFuture<Self, &'i Self> {
        NextChangeFuture::new(self)
    }
}
impl<T: Observable> ObservableExt for T {}

pin_project! {
    pub struct NextChangeFuture<I, A>
    where
        A: Borrow<I>,
        I: ObservableBase,
        I: ?Sized,
    {
        inner: A,
        start_version: Version,
        _phantom: PhantomData<Box<I>>,
    }
}
impl<I, A> NextChangeFuture<I, A>
where
    A: Borrow<I>,
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
    pub fn observable(&self) -> &A {
        &self.inner
    }
    pub fn rewind(&mut self) {
        self.start_version = self.inner.borrow().get_version();
    }
}

impl<I, A> Future for NextChangeFuture<I, A>
where
    A: Borrow<I>,
    I: ObservableBase,
    I: ?Sized,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let inner: &I = (&*this.inner).borrow();
        if this.start_version.is_null() {
            *this.start_version = inner.get_version();
            inner.add_waker(cx.waker().to_owned());
        }
        if inner.get_version() > *this.start_version {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
