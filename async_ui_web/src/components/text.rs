use std::{
    future::{Future,},
    pin::Pin,
    task::{Context, Poll},
};

use observables::{NextChangeFuture, ObservableAs, ObservableAsExt};

use crate::window::DOCUMENT;

use super::ElementFuture;

pub struct TextFuture<'c> {
    obs: &'c (dyn ObservableAs<str> + 'c),
    change_fut: NextChangeFuture<dyn ObservableAs<str> + 'c, &'c (dyn ObservableAs<str> + 'c)>,
    node: web_sys::Text,
    set: bool,
}

impl<'c> Future for TextFuture<'c> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let reset = match Pin::new(&mut this.change_fut).poll(cx) {
            Poll::Ready(_) => {
                this.change_fut = this.obs.until_change();
                let _ = Pin::new(&mut this.change_fut).poll(cx);
                true
            }
            Poll::Pending => false,
        };
        if reset || !this.set {
            this.set = true;
            let txt = this.obs.borrow_observable_as();
            this.node.set_data(&*txt);
        }
        Poll::Pending
    }
}
pub async fn text<'c>(text: &'c (dyn ObservableAs<str> + 'c)) {
    let node: web_sys::Text = DOCUMENT.with(|doc| doc.create_text_node(""));
    let fut = TextFuture {
        change_fut: NextChangeFuture::new(text),
        obs: text,
        node: node.clone(),
        set: false,
    };
    ElementFuture::new(fut, node.into()).await;
}
