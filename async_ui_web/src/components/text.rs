use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use observables::{NextChangeFuture, Observable};
use pin_project_lite::pin_project;
use wasm_bindgen::JsCast;
use web_sys::HtmlSpanElement;

use crate::window::DOCUMENT;

use super::ElementFuture;

pub struct Text<V: Observable<Data = str>> {
    pub text: V,
}

pin_project! {
    pub struct TextFuture<V: Observable<Data = str>> {
        #[pin]
        change_fut: NextChangeFuture<V, V>,
        node: HtmlSpanElement,
        set: bool
    }
}

impl<V: Observable<Data = str>> Future for TextFuture<V> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let reset = match this.change_fut.as_mut().poll(cx) {
            Poll::Ready(_) => {
                this.change_fut.rewind();
                true
            }
            Poll::Pending => false,
        };
        if reset || !*this.set {
            *this.set = true;
            this.change_fut.observable().visit(|st| {
                this.node.set_text_content(Some(st));
            });
        }
        Poll::Pending
    }
}
impl<V: Observable<Data = str>> IntoFuture for Text<V> {
    type Output = ();

    type IntoFuture = ElementFuture<TextFuture<V>>;

    fn into_future(self) -> Self::IntoFuture {
        let node: HtmlSpanElement = DOCUMENT
            .with(|doc| doc.create_element("span").expect("create element failed"))
            .unchecked_into();
        let fut = TextFuture {
            change_fut: NextChangeFuture::new(self.text),
            node: node.clone(),
            set: false,
        };
        ElementFuture::new(fut, node.into())
    }
}
