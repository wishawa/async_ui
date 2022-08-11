use std::future::Future;
use std::task::{Context, Poll};
use std::{pin::Pin, rc::Rc};

use js_sys::Function;
use observables::{cell::ObservableCell, NextChangeFuture};
use pin_project_lite::pin_project;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

pin_project! {
    pub(super) struct EventHandler<Event, Handler: FnMut(Event) = fn(Event)> {
        handler: Handler,
        closure: Closure<dyn Fn(Event)>,
        cell: Rc<ObservableCell<Option<Event>>>,
        listener: NextChangeFuture<Rc<ObservableCell<Option<Event>>>, ObservableCell<Option<Event>>>
    }
}

impl<Event: wasm_bindgen::convert::FromWasmAbi + 'static, Handler: FnMut(Event)>
    EventHandler<Event, Handler>
{
    pub fn new(handler: Handler) -> Self {
        let cell = Rc::new(ObservableCell::new(None));
        let listener = NextChangeFuture::new(cell.clone());
        let cell_cloned = cell.clone();
        let closure = Closure::new(move |event| {
            *cell_cloned.borrow_mut() = Some(event);
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
impl<Event, Handler: FnMut(Event)> Future for EventHandler<Event, Handler> {
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
            }
            Poll::Pending => {}
        }
        Poll::Pending
    }
}