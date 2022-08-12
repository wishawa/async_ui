use std::{
    cell::{Cell, RefCell},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures_signals::signal::Signal;
use waker_fn::waker_fn;

use crate::{Observable, ObservableBase, Version};

pub struct ToSignal<'i, I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
    Self: Unpin,
{
    wrapped: &'i I,
    mapper: M,
    last_version: Version,
}

impl<'i, I, O, M> ToSignal<'i, I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
    Self: Unpin,
{
    pub fn new(wrapped: &'i I, mapper: M) -> Self {
        Self {
            wrapped,
            mapper,
            last_version: Version::new_null(),
        }
    }
}

impl<'i, I, O, M> Signal for ToSignal<'i, I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
    Self: Unpin,
{
    type Item = O;

    fn poll_change(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let current_version = this.wrapped.get_version();
        if current_version > this.last_version {
            this.last_version = current_version;
            let val = this.wrapped.obs_borrow();
            let out = (this.mapper)(&*val);
            this.wrapped.add_waker(cx.waker().to_owned());
            Poll::Ready(Some(out))
        } else {
            Poll::Pending
        }
    }
}

pub struct FromSignal<S>
where
    S: Signal + Unpin,
    S::Item: Default,
{
    signal: RefCell<S>,
    value: RefCell<S::Item>,
    wakers: Arc<Mutex<Vec<Waker>>>,
    combined_waker: Waker,
    version: Cell<Version>,
}

impl<S> FromSignal<S>
where
    S: Signal + Unpin,
    S::Item: Default,
{
    pub fn new(signal: S) -> Self {
        let wakers: Arc<Mutex<Vec<Waker>>> = Default::default();
        let wakers_cloned = wakers.clone();
        Self {
            signal: RefCell::new(signal),
            value: RefCell::new(Default::default()),
            wakers,
            combined_waker: waker_fn(move || {
                wakers_cloned
                    .lock()
                    .unwrap()
                    .drain(..)
                    .for_each(Waker::wake);
            }),
            version: Cell::new(Version::new()),
        }
    }
}
impl<S> Observable for FromSignal<S>
where
    S: Signal + Unpin,
    S::Item: Default,
{
    type Data = S::Item;

    fn obs_borrow<'b>(&'b self) -> crate::ObservableBorrowed<'b, Self::Data> {
        crate::ObservableBorrowed::RefCell(self.value.borrow())
    }
}

impl<S> ObservableBase for FromSignal<S>
where
    S: Signal + Unpin,
    S::Item: Default,
{
    fn add_waker(&self, waker: Waker) {
        self.wakers.lock().unwrap().push(waker)
    }
    fn get_version(&self) -> Version {
        let this = self;
        let mut cx = Context::from_waker(&this.combined_waker);
        let mut sig = this.signal.borrow_mut();
        let sig = Pin::new(&mut *sig);
        match sig.poll_change(&mut cx) {
            Poll::Ready(Some(item)) => {
                *self.value.borrow_mut() = item;
                let new_ver = self.version.get().incremented();
                self.version.set(new_ver);
                new_ver
            }
            _ => self.version.get(),
        }
    }
}
