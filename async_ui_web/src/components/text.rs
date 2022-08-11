use std::{
    borrow::Borrow,
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

pub struct Text<V: Observable<Data = T>, T: Borrow<str> + ?Sized> {
    pub text: V,
}

pin_project! {
    pub struct TextFuture<V, T>
    where
        T: Borrow<str>,
        T: ?Sized,
        V: Observable<Data = T>
    {
        #[pin]
        change_fut: NextChangeFuture<V, V>,
        node: HtmlSpanElement,
        set: bool
    }
}

impl<V: Observable<Data = T>, T: Borrow<str> + ?Sized> Future for TextFuture<V, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let reset = match this.change_fut.as_mut().poll(cx) {
            Poll::Ready(_) => {
                this.change_fut.rewind();
                let _ = this.change_fut.as_mut().poll(cx);
                true
            }
            Poll::Pending => false,
        };
        if reset || !*this.set {
            *this.set = true;
            this.change_fut.observable().visit(|st| {
                this.node.set_text_content(Some(st.borrow()));
            });
        }
        Poll::Pending
    }
}
impl<V: Observable<Data = T>, T: Borrow<str> + ?Sized> IntoFuture for Text<V, T> {
    type Output = ();

    type IntoFuture = ElementFuture<TextFuture<V, T>>;

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
