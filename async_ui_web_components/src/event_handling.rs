use std::{
    borrow::Cow,
    cell::RefCell,
    collections::BTreeMap,
    future::Future,
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
    rc::{Rc, Weak},
    task::{Poll, Waker},
};

use futures_core::Stream;
use pin_project::{pin_project, pinned_drop};
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct WakerKey<'t>(Cow<'t, str>, usize);

pub struct EventsHandler {
    wakers: BTreeMap<WakerKey<'static>, Waker>,
    last_event: Option<(String, web_sys::Event)>,
    last_event_id: usize,
    target: web_sys::EventTarget,
    closure: Closure<dyn Fn(web_sys::Event)>,
}

impl EventsHandler {
    pub fn new(target: web_sys::EventTarget) -> Rc<RefCell<Self>> {
        Rc::new_cyclic(|weak: &Weak<RefCell<EventsHandler>>| {
            let weak = weak.to_owned();
            let closure = Closure::new(move |event: web_sys::Event| {
                {
                    if let Some(upgraded) = weak.upgrade() {
                        let mut shared = &mut *(&*upgraded).borrow_mut();
                        let ty = event.type_();
                        for (_, waker) in shared.wakers.range(
                            WakerKey(Cow::Borrowed(&ty), 0)
                                ..=WakerKey(Cow::Borrowed(&ty), usize::MAX),
                        ) {
                            waker.wake_by_ref();
                        }
                        shared.last_event_id = shared.last_event_id.wrapping_add(1);
                        shared.last_event = Some((ty, event));
                    }
                }
                async_ui_web_core::executor::run_now();
            });

            RefCell::new(Self {
                wakers: BTreeMap::new(),
                last_event: None,
                last_event_id: 0,
                target,
                closure,
            })
        })
    }
    pub fn on_event<V: JsCast>(
        this: Rc<RefCell<Self>>,
        ev_type: Cow<'static, str>,
    ) -> NextEvent<V> {
        NextEvent {
            ev_type,
            shared: this,
            last_event_id: 0,
            put_waker: None,
            _ty: (PhantomData, PhantomPinned),
        }
    }
}

#[pin_project(PinnedDrop)]
pub struct NextEvent<V: JsCast> {
    ev_type: Cow<'static, str>,
    shared: Rc<RefCell<EventsHandler>>,
    last_event_id: usize,
    put_waker: Option<Waker>,
    _ty: (PhantomData<V>, PhantomPinned),
}

impl<V: JsCast> Stream for NextEvent<V> {
    type Item = V;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<V>> {
        let self_addr = &*self as *const Self as usize;
        let this = self.project();
        let shared = &mut this.shared.borrow_mut();
        if shared.last_event_id == *this.last_event_id {
            return Poll::Pending;
        }
        *this.last_event_id = shared.last_event_id;
        let new_waker = cx.waker();
        match this.put_waker {
            Some(w) => {
                let out = match shared.last_event.as_ref() {
                    Some((ty, e)) if *ty == *this.ev_type => {
                        let v = e.clone().unchecked_into();
                        Poll::Ready(Some(v))
                    }
                    _ => Poll::Pending,
                };
                if !w.will_wake(new_waker) {
                    shared.wakers.insert(
                        WakerKey(this.ev_type.clone(), self_addr),
                        new_waker.to_owned(),
                    );
                    *this.put_waker = Some(new_waker.to_owned());
                }
                out
            }
            _ => {
                shared.wakers.insert(
                    WakerKey(this.ev_type.clone(), self_addr),
                    new_waker.to_owned(),
                );
                *this.put_waker = Some(new_waker.to_owned());
                Poll::Pending
            }
        }
    }
}

impl<V: JsCast> Future for NextEvent<V> {
    type Output = V;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.poll_next(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(v),
            _ => Poll::Pending,
        }
    }
}

#[pinned_drop]
impl<V: JsCast> PinnedDrop for NextEvent<V> {
    fn drop(self: Pin<&mut Self>) {
        let self_addr = &*self as *const Self as usize;
        let this = self.project();
        let mut shared = this.shared.borrow_mut();
        if this.put_waker.is_some() {
            let ty = std::mem::replace(this.ev_type, Cow::Borrowed(""));
            let wk = WakerKey(ty, self_addr);
            shared.wakers.remove(&wk);
            let ty = wk.0;
            if shared
                .wakers
                .range(WakerKey(Cow::Borrowed(&ty), 0)..=WakerKey(Cow::Borrowed(&ty), usize::MAX))
                .next()
                .is_none()
            {
                shared
                    .target
                    .remove_event_listener_with_callback(
                        &ty,
                        shared.closure.as_ref().unchecked_ref(),
                    )
                    .unwrap_throw();
            }
        }
    }
}
