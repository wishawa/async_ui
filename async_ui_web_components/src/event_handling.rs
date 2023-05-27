use std::{borrow::Cow, cell::RefCell, future::Future, rc::Rc, task::Poll};

use futures_core::Stream;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{AddEventListenerOptions, EventTarget};

pub struct NextEvent<E> {
    target: EventTarget,
    closure: Option<Closure<dyn Fn(web_sys::Event)>>,
    shared: Rc<RefCell<Option<E>>>,
    options: Option<AddEventListenerOptions>,
    event_name: Cow<'static, str>,
}

impl<E: JsCast> NextEvent<E> {
    pub fn new(target: EventTarget, event_name: Cow<'static, str>) -> Self {
        let shared = Rc::new(RefCell::new(None));
        Self {
            target,
            closure: None,
            shared,
            options: None,
            event_name,
        }
    }
    pub fn set_capture(&mut self, capture: bool) {
        self.options
            .get_or_insert_with(AddEventListenerOptions::new)
            .capture(capture);
    }
    pub fn set_passive(&mut self, passive: bool) {
        self.options
            .get_or_insert_with(AddEventListenerOptions::new)
            .passive(passive);
    }
}

impl<E: JsCast + 'static> Future for NextEvent<E> {
    type Output = E;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.poll_next(cx) {
            Poll::Ready(Some(ev)) => Poll::Ready(ev),
            _ => Poll::Pending,
        }
    }
}
impl<E: JsCast + 'static> Stream for NextEvent<E> {
    type Item = E;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.closure.is_none() {
            let waker = cx.waker().to_owned();
            let shared_weak = Rc::downgrade(&this.shared);
            let closure = Closure::new(move |ev: web_sys::Event| {
                if let Some(strong) = shared_weak.upgrade() {
                    let inner = &mut *strong.borrow_mut();
                    *inner = Some(ev.unchecked_into());
                    waker.wake_by_ref();
                }
                async_ui_web_core::executor::run_now();
            });
            let listener = closure.as_ref().unchecked_ref();
            if let Some(options) = &this.options {
                this.target
                    .add_event_listener_with_callback_and_add_event_listener_options(
                        &this.event_name,
                        listener,
                        options,
                    )
                    .unwrap_throw();
            } else {
                this.target
                    .add_event_listener_with_callback(&this.event_name, listener)
                    .unwrap_throw();
            }
            this.closure = Some(closure);
            Poll::Pending
        } else if let Some(ev) = this.shared.borrow_mut().take() {
            Poll::Ready(Some(ev))
        } else {
            Poll::Pending
        }
    }
}

pub trait EmitEvent {
    fn until_event<E: JsCast + 'static>(&self, name: Cow<'static, str>) -> NextEvent<E>;
}

impl EmitEvent for EventTarget {
    fn until_event<E: JsCast + 'static>(&self, name: Cow<'static, str>) -> NextEvent<E> {
        NextEvent::new(self.to_owned(), name)
    }
}

impl<E> Drop for NextEvent<E> {
    fn drop(&mut self) {
        if let Some(callback) = self.closure.take() {
            self.target
                .remove_event_listener_with_callback(
                    &self.event_name,
                    callback.as_ref().unchecked_ref(),
                )
                .unwrap_throw();
        }
    }
}
