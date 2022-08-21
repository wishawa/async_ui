use std::{
    borrow::Borrow,
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use observables::{NextChangeFuture, ObservableAs, ObservableAsExt};

use crate::window::DOCUMENT;

use super::ElementFuture;

pub struct Text<'c> {
    pub text: &'c (dyn ObservableAs<str> + 'c),
}

impl<'c> Default for Text<'c> {
    fn default() -> Self {
        Self { text: &"" }
    }
}

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
            this.node.set_data((&*txt).borrow());
        }
        Poll::Pending
    }
}
impl<'c> IntoFuture for Text<'c> {
    type Output = ();

    type IntoFuture = ElementFuture<TextFuture<'c>>;

    fn into_future(self) -> Self::IntoFuture {
        let node: web_sys::Text = DOCUMENT.with(|doc| doc.create_text_node(""));
        let fut = TextFuture {
            change_fut: NextChangeFuture::new(self.text),
            obs: self.text,
            node: node.clone(),
            set: false,
        };
        ElementFuture::new(fut, node.into())
    }
}
