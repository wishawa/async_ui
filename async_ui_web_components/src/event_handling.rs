use std::{
    borrow::Cow,
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::{Poll, Waker},
};

use async_ui_internal_utils::dummy_waker::dummy_waker;
use futures_core::Stream;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{AddEventListenerOptions, EventTarget};

/// A struct implementing both [Future] and [Stream].
/// Yields [Event][web_sys::Event] objects.
///
/// Use [until_event][crate::events::EmitEvent::until_event] or other until_*
/// methods to get this struct.
///
/// The implementation only keeps the last event it receives.
/// This means if you use some custom manually-implemented wrapper futures and
/// fail to poll the stream upon `wake`, you might miss events.
pub struct EventFutureStream<E> {
    target: EventTarget,
    closure: Option<Closure<dyn Fn(web_sys::Event)>>,
    shared: Rc<RefCell<(Option<E>, Waker)>>,
    options: Option<AddEventListenerOptions>,
    event_name: Cow<'static, str>,
}

impl<E: JsCast> EventFutureStream<E> {
    /// Prefer to use [until_event][crate::events::EmitEvent::until_event] or other until_*
    /// methods instead of this.
    pub fn new(target: EventTarget, event_name: Cow<'static, str>) -> Self {
        Self {
            target,
            closure: None,
            shared: Rc::new(RefCell::new((None, dummy_waker()))),
            options: None,
            event_name,
        }
    }
    /// The `capture` option indicates that that events of this type will be
    /// dispatched to the registered listener before being dispatched to any
    /// EventTarget beneath it in the DOM tree.
    /// If not specified, defaults to false.
    ///
    /// See [MDN documentation on `addEventListener`](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener).
    ///
    /// This needs to be set *before* you first poll the stream.
    pub fn set_capture(&mut self, capture: bool) {
        self.options
            .get_or_insert_with(AddEventListenerOptions::new)
            .capture(capture);
    }
    /// The `passive` option indicates that the function specified by listener
    /// will never call `preventDefault()`.
    /// If a passive listener does call `preventDefault()`,
    /// the user agent will do nothing other than generate a console warning.
    /// If not specified, defaults to false.
    ///
    /// See [MDN documentation on `addEventListener`](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener).
    /// See [prevent_default][web_sys::Event::prevent_default].
    ///
    /// This needs to be set *before* you first poll the stream.
    pub fn set_passive(&mut self, passive: bool) {
        self.options
            .get_or_insert_with(AddEventListenerOptions::new)
            .passive(passive);
    }
}

impl<E: JsCast + 'static> Future for EventFutureStream<E> {
    type Output = E;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.poll_next(cx) {
            Poll::Ready(Some(ev)) => Poll::Ready(ev),
            _ => Poll::Pending,
        }
    }
}
impl<E: JsCast + 'static> Stream for EventFutureStream<E> {
    type Item = E;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        {
            let shared = &mut *this.shared.borrow_mut();
            let waker = cx.waker();
            if !shared.1.will_wake(waker) {
                shared.1 = waker.to_owned();
            }
        }

        if this.closure.is_none() {
            let shared_weak = Rc::downgrade(&this.shared);
            let closure = Closure::new(move |ev: web_sys::Event| {
                if let Some(strong) = shared_weak.upgrade() {
                    let inner = &mut *strong.borrow_mut();
                    inner.0 = Some(ev.unchecked_into());
                    inner.1.wake_by_ref();
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
        } else if let Some(ev) = this.shared.borrow_mut().0.take() {
            Poll::Ready(Some(ev))
        } else {
            Poll::Pending
        }
    }
}

/// Implemented for [EventTarget].
/// ```
/// # use async_ui_web_components::events::EmitEvent;
/// # let _ = async {
/// # let event_target = web_sys::EventTarget::new().unwrap();
/// let _ev = event_target.until_event::<web_sys::Event>("eventname".into()).await;
/// // do something after event
/// # };
/// ```
pub trait EmitEvent {
    /// Wait until an event with the specified name is fired.
    /// The return type is both a [Future] and a [Stream] that yields the event object.
    fn until_event<E: JsCast + 'static>(&self, name: Cow<'static, str>) -> EventFutureStream<E>;
}

impl EmitEvent for EventTarget {
    fn until_event<E: JsCast + 'static>(&self, name: Cow<'static, str>) -> EventFutureStream<E> {
        EventFutureStream::new(self.to_owned(), name)
    }
}

impl<E> Drop for EventFutureStream<E> {
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
