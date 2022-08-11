use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, MouseEvent};

use crate::{window::DOCUMENT, Render};

use super::{event_handler::EventHandler, ElementFuture};

pub struct Button<'c, OnPress: FnMut(MouseEvent) = fn(MouseEvent)> {
    pub children: Render<'c>,
    pub on_press: OnPress,
}

pin_project! {
    pub struct ButtonFuture<'c, OnPress: FnMut(MouseEvent)> {
        #[pin]
        children: Render<'c>,
        #[pin]
        on_press: EventHandler<MouseEvent, OnPress>
    }
}
impl<'c, OnPress: FnMut(MouseEvent)> Future for ButtonFuture<'c, OnPress> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _ = this.on_press.poll(cx);
        this.children.poll(cx)
    }
}
impl<'c, OnPress: FnMut(MouseEvent)> IntoFuture for Button<'c, OnPress> {
    type Output = ();
    type IntoFuture = ElementFuture<ButtonFuture<'c, OnPress>>;

    fn into_future(self) -> Self::IntoFuture {
        let button = DOCUMENT.with(|doc| {
            let elem = doc.create_element("button").expect("create element failed");
            let elem: HtmlButtonElement = elem.unchecked_into();
            elem
        });
        let on_press = EventHandler::new(self.on_press);
        let onclick_function = on_press.get_function();
        button.set_onclick(Some(onclick_function));
        let future = ButtonFuture {
            children: self.children,
            on_press,
        };
        ElementFuture {
            node: button.into(),
            future,
            vnode: None,
        }
    }
}
