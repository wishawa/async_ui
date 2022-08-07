use std::{
    cell::{Cell, RefCell},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures_signals::signal::Signal;
use pin_cell::{PinCell, PinMut};
use pin_project_lite::pin_project;
use waker_fn::waker_fn;

use crate::{Observable, ObservableBase};

pin_project! {
    pub struct ToSignal<I, O, M>
    where
        I: Observable,
        M: Fn(&I::Data) -> O,
    {
        #[pin]
        wrapped: I,
        mapper: M,
        last_version: u64
    }
}

impl<I, O, M> Signal for ToSignal<I, O, M>
where
    I: Observable,
    M: Fn(&I::Data) -> O,
{
    type Item = O;

    fn poll_change(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let current_version = this.wrapped.as_ref().get_version();
        if current_version > *this.last_version {
            *this.last_version = current_version;
            let out = this.wrapped.visit(|input| {
                let output = (this.mapper)(input);
                output
            });
            this.wrapped.as_ref().add_waker(cx.waker().to_owned());
            Poll::Ready(Some(out))
        } else {
            Poll::Pending
        }
    }
}

pin_project! {
    pub struct FromSignal<S>
    where
        S: Signal,
        S::Item: Default
    {
        #[pin]
        signal: PinCell<S>,
        value: RefCell<Option<S::Item>>,
        wakers: Arc<Mutex<Vec<Waker>>>,
        combined_waker: Waker,
        version: Cell<u64>,
    }
}

impl<S> FromSignal<S>
where
    S: Signal,
    S::Item: Default,
{
    pub fn new(signal: S) -> Self {
        let wakers: Arc<Mutex<Vec<Waker>>> = Default::default();
        let wakers_cloned = wakers.clone();
        Self {
            signal: PinCell::new(signal),
            value: RefCell::new(None),
            wakers,
            combined_waker: waker_fn(move || {
                wakers_cloned
                    .lock()
                    .unwrap()
                    .drain(..)
                    .for_each(Waker::wake);
            }),
            version: Cell::new(0),
        }
    }
}
impl<S> Observable for FromSignal<S>
where
    S: Signal,
    S::Item: Default,
{
    type Data = S::Item;
    fn visit<R, F: FnOnce(&Self::Data) -> R>(&self, func: F) -> R {
        let mut bm = self.value.borrow_mut();
        let value = bm.get_or_insert_with(Default::default);
        func(value)
    }
}

impl<S> ObservableBase for FromSignal<S>
where
    S: Signal,
    S::Item: Default,
{
    fn add_waker(self: Pin<&Self>, waker: Waker) {
        self.wakers.lock().unwrap().push(waker)
    }
    fn get_version(self: Pin<&Self>) -> u64 {
        let this = self.project_ref();
        let mut cx = Context::from_waker(&this.combined_waker);
        let mut sig = this.signal.borrow_mut();
        let sig = PinMut::as_mut(&mut sig);
        match sig.poll_change(&mut cx) {
            Poll::Ready(Some(item)) => {
                *self.value.borrow_mut() = Some(item);
                let new_ver = self.version.get() + 1;
                self.version.set(new_ver);
                new_ver
            }
            _ => self.version.get(),
        }
    }
}
