use crate::WrappedWidget;
use std::{
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::{Poll, Waker},
};

macro_rules! make_connect {
	($widget:ty, $connect_name:ident, $($uses:path),*) => {
		paste::paste! {
			impl<'a> WrappedWidget<'a, $widget> {
				pub fn [<on_ $connect_name>]<C: FnMut(&$widget) + 'a>(mut self, mut callback: C) -> Self {
					$(
						use $uses;
					)*
					let (tx, rx) = create_channel::<()>();
					self.widget.[<connect_ $connect_name>](move |_| {
						tx.send(());
					});
					let wg = self.widget.clone();
					self.asyncs.push(Box::pin(async move {
						let widget = wg;
						let recv = rx;
						loop {
							let _ev = recv.clone().await;
							callback(&widget);
						}
					}));
					self
				}
			}
		}
	};
}
make_connect!(gtk::Button, clicked, gtk::prelude::ButtonExt);
make_connect!(gtk::Entry, changed, gtk::prelude::EditableExt);
make_connect!(gtk::Entry, activate, gtk::prelude::EntryExt);
make_connect!(gtk::SpinButton, value_changed,);

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
