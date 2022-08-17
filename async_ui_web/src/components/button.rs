use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use pin_project_lite::pin_project;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, MouseEvent};

use crate::{window::DOCUMENT, Fragment};

use super::{
    dummy::{dummy_handler, is_dummy},
    event_handler::EventHandler,
    ElementFuture,
};

pub struct PressEvent {
    pub native_event: MouseEvent,
}
pub struct Button<'c> {
    pub children: Fragment<'c>,
    pub on_press: &'c (dyn Fn(PressEvent) + 'c),
    pub on_press_in: &'c (dyn Fn(PressEvent) + 'c),
    pub on_press_out: &'c (dyn Fn(PressEvent) + 'c),
}

impl<'c> Default for Button<'c> {
    fn default() -> Self {
        Self {
            children: Default::default(),
            on_press: &dummy_handler,
            on_press_in: &dummy_handler,
            on_press_out: &dummy_handler,
        }
    }
}

pin_project! {
    pub struct ButtonFuture<'c> {
        #[pin] children: Fragment<'c>,
        on_press: Option<(EventHandler<'c, MouseEvent>, &'c (dyn Fn(PressEvent) + 'c))>,
        on_press_in: Option<(EventHandler<'c, MouseEvent>, &'c (dyn Fn(PressEvent) + 'c))>,
        on_press_out: Option<(EventHandler<'c, MouseEvent>, &'c (dyn Fn(PressEvent) + 'c))>,
    }
}
impl<'c> Future for ButtonFuture<'c> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _ = this.on_press.as_mut().map(|(listener, handler)| {
            match Pin::new(listener).poll_next(cx) {
                Poll::Ready(Some(ev)) => handler(PressEvent { native_event: ev }),
                _ => (),
            }
        });
        let _ = this.on_press_in.as_mut().map(|(listener, handler)| {
            match Pin::new(listener).poll_next(cx) {
                Poll::Ready(Some(ev)) => handler(PressEvent { native_event: ev }),
                _ => (),
            }
        });
        let _ = this.on_press_out.as_mut().map(|(listener, handler)| {
            match Pin::new(listener).poll_next(cx) {
                Poll::Ready(Some(ev)) => handler(PressEvent { native_event: ev }),
                _ => (),
            }
        });
        this.children.poll(cx)
    }
}
impl<'c> IntoFuture for Button<'c> {
    type Output = ();
    type IntoFuture = ElementFuture<ButtonFuture<'c>>;

    fn into_future(self) -> Self::IntoFuture {
        let button = DOCUMENT.with(|doc| {
            let elem = doc.create_element("button").expect("create element failed");
            let elem: HtmlButtonElement = elem.unchecked_into();
            elem
        });
        let on_press = (!is_dummy(self.on_press)).then(|| {
            let listener = EventHandler::new();
            button.set_onclick(Some(listener.get_function()));
            (listener, self.on_press)
        });
        let on_press_in = (!is_dummy(self.on_press_in)).then(|| {
            let listener = EventHandler::new();
            button.set_onpointerdown(Some(listener.get_function()));
            (listener, self.on_press_in)
        });
        let on_press_out = (!is_dummy(self.on_press_out)).then(|| {
            let listener = EventHandler::new();
            button.set_onpointerup(Some(listener.get_function()));
            (listener, self.on_press_out)
        });

        let future = ButtonFuture {
            children: self.children,
            on_press,
            on_press_in,
            on_press_out,
        };
        ElementFuture::new(future, button.into())
    }
}
