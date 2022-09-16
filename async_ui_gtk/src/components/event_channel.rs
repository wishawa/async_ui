use std::marker::PhantomData;
use std::task::{Context, Poll};
use std::{pin::Pin, rc::Rc};

use futures::Stream;
use observables::cell::ObservableCell;
use observables::{Listenable, Version};

pub(super) struct EventHandler<'h, E> {
    cell: Rc<ObservableCell<Option<E>>>,
    last_version: Version,
    phantom: PhantomData<&'h ()>,
}
pub(super) struct EventReceiver<E> {
    cell: Rc<ObservableCell<Option<E>>>,
}
impl<E: 'static> EventReceiver<E> {
    pub fn send(&self, value: E) {
        *self.cell.borrow_mut() = Some(value);
    }
}

impl<'h, E: 'static> EventHandler<'h, E> {
    pub fn new() -> Self {
        let cell = Rc::new(ObservableCell::new(None));

        let last_version = Version::new_null();
        Self {
            cell,
            last_version,
            phantom: PhantomData,
        }
    }
    pub fn get_receiver(&self) -> EventReceiver<E> {
        EventReceiver {
            cell: self.cell.clone(),
        }
    }
}
impl<'h, E: 'static> Stream for EventHandler<'h, E> {
    type Item = E;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let obs = this.cell.as_observable();
        let current_version = obs.get_version();
        if this.last_version.is_null() {
            this.last_version = current_version;
            obs.add_waker(cx.waker().to_owned());
        }
        if current_version > this.last_version {
            let res = {
                if let Some(event) = this.cell.borrow_mut().take() {
                    Poll::Ready(Some(event))
                } else {
                    Poll::Pending
                }
            };
            this.last_version = obs.get_version();
            obs.add_waker(cx.waker().to_owned());
            res
        } else {
            Poll::Pending
        }
    }
}
