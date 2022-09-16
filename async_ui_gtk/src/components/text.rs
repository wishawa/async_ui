use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use glib::Cast;
use observables::{NextChangeFuture, ObservableAs, ObservableAsExt};

use crate::widget::WrappedWidget;

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
    node: gtk::Label,
    obs: &'c (dyn ObservableAs<str> + 'c),
    change_fut: NextChangeFuture<dyn ObservableAs<str> + 'c, &'c (dyn ObservableAs<str> + 'c)>,
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
            this.node.set_label(&*txt)
        }
        Poll::Pending
    }
}

impl<'c> IntoFuture for Text<'c> {
    type IntoFuture = ElementFuture<TextFuture<'c>>;
    type Output = ();
    fn into_future(self) -> Self::IntoFuture {
        let node = gtk::Label::new(None);
        let tf = TextFuture {
            change_fut: NextChangeFuture::new(self.text),
            obs: self.text,
            node: node.clone(),
            set: false,
        };
        ElementFuture::new(
            tf,
            WrappedWidget {
                widget: node.upcast(),
                inner_widget: None,
                op: crate::widget::WidgetOp::NoChild,
            },
        )
    }
}
