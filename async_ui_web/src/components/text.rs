use std::{
    borrow::Borrow,
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use observables::{NextChangeFuture, Observable};

use crate::window::DOCUMENT;

use super::ElementFuture;

pub struct Text<'c, T: Borrow<str> + ?Sized> {
    pub text: &'c (dyn Observable<Data = T> + 'c),
}

pub struct TextFuture<'c, T: Borrow<str> + ?Sized> {
    change_fut:
        NextChangeFuture<dyn Observable<Data = T> + 'c, &'c (dyn Observable<Data = T> + 'c)>,
    node: web_sys::Text,
    set: bool,
}

impl<'c, T: Borrow<str> + ?Sized> Future for TextFuture<'c, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let reset = match Pin::new(&mut this.change_fut).poll(cx) {
            Poll::Ready(_) => {
                this.change_fut.rewind();
                let _ = Pin::new(&mut this.change_fut).poll(cx);
                true
            }
            Poll::Pending => false,
        };
        if reset || !this.set {
            this.set = true;
            let txt = this.change_fut.observable().get_borrow();
            this.node.set_data((&*txt).borrow());
        }
        Poll::Pending
    }
}
impl<'c, T: Borrow<str> + ?Sized> IntoFuture for Text<'c, T> {
    type Output = ();

    type IntoFuture = ElementFuture<TextFuture<'c, T>>;

    fn into_future(self) -> Self::IntoFuture {
        let node: web_sys::Text = DOCUMENT.with(|doc| doc.create_text_node(""));
        let fut = TextFuture {
            change_fut: NextChangeFuture::new(self.text),
            node: node.clone(),
            set: false,
        };
        ElementFuture::new(fut, node.into())
    }
}
