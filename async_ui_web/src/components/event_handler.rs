use std::future::Future;
use std::task::{Context, Poll};
use std::{pin::Pin, rc::Rc};

use js_sys::Function;
use observables::cell::ObservableCell;
use observables::{ObservableBase, Version};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use crate::executor::schedule;

pub(super) struct EventHandler<'h, E> {
    handler: &'h (dyn Fn(E) + 'h),
    closure: Closure<dyn Fn(E) + 'h>,
    cell: Rc<ObservableCell<Option<E>>>,
    last_version: Version,
}

impl<'h, E: wasm_bindgen::convert::FromWasmAbi + 'static> EventHandler<'h, E> {
    pub fn new(handler: &'h dyn Fn(E)) -> Self {
        let cell = Rc::new(ObservableCell::new(None));
        let cell_cloned = cell.clone();
        let closure = Closure::new(move |event| {
            *cell_cloned.borrow_mut() = Some(event);
            schedule();
        });
        let last_version = Version::new_null();
        Self {
            handler,
            cell,
            closure,
            last_version,
        }
    }
    pub fn get_function(&self) -> &Function {
        self.closure.as_ref().unchecked_ref()
    }
}
impl<'h, Event> Future for EventHandler<'h, Event> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let obs = this.cell.as_observable();
        let current_version = obs.get_version();
        if this.last_version.is_null() {
            this.last_version = current_version;
            obs.add_waker(cx.waker().to_owned());
        }
        if current_version > this.last_version {
            {
                if let Some(event) = this.cell.borrow_mut().take() {
                    (this.handler)(event);
                }
            }
            this.last_version = obs.get_version();
            obs.add_waker(cx.waker().to_owned());
        }
        Poll::Pending
    }
}
