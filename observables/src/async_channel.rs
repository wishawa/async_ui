use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use ::async_channel::Receiver;
use waker_fn::waker_fn;

use crate::{Listenable, ObservableBase, Version};

pub struct FromReceiver<T> {
    receiver: Receiver<T>,
    last_value: RefCell<T>,
    wakers: Arc<Mutex<Vec<Waker>>>,
    combined_waker: Waker,
    version: Cell<Version>,
}

impl<T> Listenable for FromReceiver<T> {
    fn add_waker(&self, waker: Waker) {
        self.wakers.lock().unwrap().push(waker);
    }

    fn get_version(&self) -> Version {
        let mut cx = Context::from_waker(&self.combined_waker);
        let mut recv = self.receiver.recv();
        match Pin::new(&mut recv).poll(&mut cx) {
            Poll::Ready(Ok(value)) => {
                *self.last_value.borrow_mut() = value;
                let new_ver = self.version.get().incremented();
                self.version.set(new_ver);
                new_ver
            }
            _ => self.version.get(),
        }
    }
}

impl<T> ObservableBase for FromReceiver<T> {
    type Data = T;

    fn borrow_observable<'b>(&'b self) -> crate::ObservableBorrow<'b, Self::Data> {
        crate::ObservableBorrow::RefCell(self.last_value.borrow())
    }
}

impl<T: Default> FromReceiver<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        let wakers: Arc<Mutex<Vec<Waker>>> = Default::default();
        let wakers_cloned = wakers.clone();
        Self {
            receiver,
            wakers,
            combined_waker: waker_fn(move || {
                wakers_cloned
                    .lock()
                    .unwrap()
                    .drain(..)
                    .for_each(Waker::wake)
            }),
            version: Cell::new(Version::new()),
            last_value: RefCell::new(Default::default()),
        }
    }
}

pub trait ObservableFromChannel<T> {
    fn as_observable(self) -> FromReceiver<T>;
}
impl<T: Default> ObservableFromChannel<T> for Receiver<T> {
    fn as_observable(self) -> FromReceiver<T> {
        FromReceiver::new(self)
    }
}
