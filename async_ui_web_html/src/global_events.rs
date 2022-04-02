use crate::elem::{Elem, HtmlTag};
use async_ui_core::backend::{Backend, Spawner};
use async_ui_web::manual_apis::WebBackend;
use futures::Future;
use js_sys::Function;
use std::{
    cell::RefCell,
    rc::Rc,
    task::{Poll, Waker},
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlElement;

macro_rules! impl_event_handler {
    ($name:ident, $evtype:ty) => {
        paste::paste! {
            impl<'a, H: HtmlTag + 'a> Elem<'a, H>
            {
                pub fn [<$name:snake>]<F: FnMut(web_sys::$evtype) + 'a>(mut self, mut callback: F) -> Self {
                    let (tx, rx) = create_channel();
                    let clos: Closure<dyn FnMut(_)> = Closure::wrap(Box::new(move |e: web_sys::$evtype| {
                        tx.send(e);
                        <<WebBackend as Backend>::Spawner as Spawner>::wake_now();
                    }) as Box<dyn FnMut(_)>);
                    let func: &Function = clos.as_ref().unchecked_ref();
                    let elem: &HtmlElement = self.elem.as_ref();
                    elem.[<set_ $name:lower>](Some(func));
                    self.asyncs.push(Box::pin(async move {
                        let _clos = clos;
                        let recv = rx;
                        loop {
                            let ev = recv.clone().await;
                            callback(ev);
                        }
                    }));
                    self
                }
            }
        }
    };
}
type ChannelInner<E> = Rc<RefCell<(Option<E>, Option<Waker>)>>;
#[derive(Clone)]
struct ChannelRx<E> {
    inner: ChannelInner<E>,
}
struct ChannelTx<E> {
    inner: ChannelInner<E>,
}
impl<E> ChannelTx<E> {
    fn send(&self, val: E) {
        let mut bm = self.inner.borrow_mut();
        bm.0 = Some(val);
        if let Some(waker) = bm.1.take() {
            waker.wake()
        }
    }
}
fn create_channel<E>() -> (ChannelTx<E>, ChannelRx<E>) {
    let inner = Rc::new(RefCell::new((None, None)));
    (
        ChannelTx {
            inner: inner.clone(),
        },
        ChannelRx { inner },
    )
}
impl<E> Future for ChannelRx<E> {
    type Output = E;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut bm = self.inner.borrow_mut();
        if let Some(v) = bm.0.take() {
            Poll::Ready(v)
        } else {
            if bm.1.is_none() {
                bm.1 = Some(cx.waker().to_owned());
            }
            Poll::Pending
        }
    }
}
// Sources:
// https://developer.mozilla.org/en-US/docs/Web/API/GlobalEventHandlers
// https://github.com/DefinitelyTyped/DefinitelyTyped/blob/f96ee5b85fd3414e8af20eb80a825a65966dd5c9/types/react/index.d.ts#L1346

impl_event_handler!(OnClick, MouseEvent);
impl_event_handler!(OnDblClick, MouseEvent);
impl_event_handler!(OnAuxClick, MouseEvent);
impl_event_handler!(OnMouseDown, MouseEvent);
impl_event_handler!(OnMouseUp, MouseEvent);
impl_event_handler!(OnMouseMove, MouseEvent);
impl_event_handler!(OnMouseEnter, MouseEvent);
impl_event_handler!(OnMouseLeave, MouseEvent);
impl_event_handler!(OnMouseOver, MouseEvent);
impl_event_handler!(OnMouseOut, MouseEvent);

impl_event_handler!(OnKeyDown, KeyboardEvent);
impl_event_handler!(OnKeyUp, KeyboardEvent);

impl_event_handler!(OnFocus, FocusEvent);
impl_event_handler!(OnBlur, FocusEvent);

impl_event_handler!(OnScroll, UiEvent);

impl_event_handler!(OnWheel, WheelEvent);

impl_event_handler!(OnInput, InputEvent);
