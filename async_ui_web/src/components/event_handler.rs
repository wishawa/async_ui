use std::future::Future;
use std::task::{Context, Poll};
use std::{pin::Pin, rc::Rc};

use js_sys::Function;
use observables::{cell::ObservableCell, NextChangeFuture};
use pin_project_lite::pin_project;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use crate::executor::schedule;

pin_project! {
    pub(super) struct EventHandler<'h, E> {
        handler: &'h (dyn Fn(E) + 'h),
        closure: Closure<dyn Fn(E) + 'h>,
        cell: Rc<ObservableCell<Option<E>>>,
        listener: NextChangeFuture<ObservableCell<Option<E>>, Rc<ObservableCell<Option<E>>>>
    }
}

impl<'h, E: wasm_bindgen::convert::FromWasmAbi + 'static> EventHandler<'h, E> {
    pub fn new(handler: &'h dyn Fn(E)) -> Self {
        let cell = Rc::new(ObservableCell::new(None));
        let listener = NextChangeFuture::new(cell.clone());
        let cell_cloned = cell.clone();
        let closure = Closure::new(move |event| {
            *cell_cloned.borrow_mut() = Some(event);
            schedule();
        });
        Self {
            handler,
            cell,
            closure,
            listener,
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
        match Pin::new(&mut this.listener).poll(cx) {
            Poll::Ready(_) => {
                {
                    if let Some(event) = this.cell.borrow_mut().take() {
                        (this.handler)(event);
                    }
                }
                this.listener = NextChangeFuture::new(this.cell.clone());
                let _ = Pin::new(&mut this.listener).poll(cx);
            }
            Poll::Pending => {}
        }
        Poll::Pending
    }
}
