use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, MouseEvent};

use crate::{window::DOCUMENT, Fragment};

use super::{event_handler::EventHandler, ElementFuture};

pub struct Button<'c> {
    pub children: Fragment<'c>,
    pub on_press: &'c dyn Fn(MouseEvent),
    pub on_press_in: &'c dyn Fn(MouseEvent),
    pub on_press_out: &'c dyn Fn(MouseEvent),
}

fn dummy_handler_fn(_me: MouseEvent) {}
impl<'c> Default for Button<'c> {
    fn default() -> Self {
        Self {
            children: Default::default(),
            on_press: &dummy_handler_fn,
            on_press_in: &dummy_handler_fn,
            on_press_out: &dummy_handler_fn,
        }
    }
}

pin_project! {
    pub struct ButtonFuture<'c> {
        #[pin]
        children: Fragment<'c>,
        #[pin]
        on_press: Option<EventHandler<'c, MouseEvent>>,
        #[pin]
        on_press_in: Option<EventHandler<'c, MouseEvent>>,
        #[pin]
        on_press_out: Option<EventHandler<'c, MouseEvent>>,
    }
}
impl<'c> Future for ButtonFuture<'c> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _ = this.on_press.as_pin_mut().map(|p| p.poll(cx));
        let _ = this.on_press_in.as_pin_mut().map(|p| p.poll(cx));
        let _ = this.on_press_out.as_pin_mut().map(|p| p.poll(cx));
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
        let on_press = (!std::ptr::eq(self.on_press as *const _, &dummy_handler_fn as *const _))
            .then(|| {
                let on_press = EventHandler::new(self.on_press);
                button.set_onclick(Some(on_press.get_function()));
                on_press
            });
        let on_press_in =
            (!std::ptr::eq(self.on_press_in as *const _, &dummy_handler_fn as *const _)).then(
                || {
                    let on_press = EventHandler::new(self.on_press);
                    button.set_onpointerdown(Some(on_press.get_function()));
                    on_press
                },
            );
        let on_press_out =
            (!std::ptr::eq(self.on_press_out as *const _, &dummy_handler_fn as *const _)).then(
                || {
                    let on_press = EventHandler::new(self.on_press);
                    button.set_onpointerup(Some(on_press.get_function()));
                    on_press
                },
            );

        let future = ButtonFuture {
            children: self.children,
            on_press,
            on_press_in,
            on_press_out,
        };
        ElementFuture {
            node: button.into(),
            future,
            vnode: None,
        }
    }
}
