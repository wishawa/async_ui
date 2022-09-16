use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use glib::Cast;
use gtk::traits::ButtonExt;
use pin_project_lite::pin_project;

use super::event_channel::EventHandler;
use crate::{
    widget::{single::ButtonOp, WrappedWidget},
    Fragment,
};

use super::{
    dummy::{create_dummy, is_dummy},
    ElementFuture,
};

pub struct PressEvent {}
pub struct Button<'c> {
    pub children: Fragment<'c>,
    pub on_press: &'c mut (dyn FnMut(PressEvent) + 'c),
    pub on_press_in: &'c mut (dyn FnMut(PressEvent) + 'c),
    pub on_press_out: &'c mut (dyn FnMut(PressEvent) + 'c),
}

impl<'c> Default for Button<'c> {
    fn default() -> Self {
        Self {
            children: Default::default(),
            on_press: create_dummy(),
            on_press_in: create_dummy(),
            on_press_out: create_dummy(),
        }
    }
}

pin_project! {
    pub struct ButtonFuture<'c> {
        #[pin] children: Fragment<'c>,
        on_press: Option<(EventHandler<'c, ()>, &'c mut (dyn FnMut(PressEvent) + 'c))>,
        on_press_in: Option<(EventHandler<'c, ()>, &'c mut (dyn FnMut(PressEvent) + 'c))>,
        on_press_out: Option<(EventHandler<'c, ()>, &'c mut (dyn FnMut(PressEvent) + 'c))>,
    }
}
impl<'c> Future for ButtonFuture<'c> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _ = this.on_press.as_mut().map(|(listener, handler)| {
            match Pin::new(listener).poll_next(cx) {
                Poll::Ready(Some(_ev)) => handler(PressEvent {}),
                _ => (),
            }
        });
        let _ = this.on_press_in.as_mut().map(|(listener, handler)| {
            match Pin::new(listener).poll_next(cx) {
                Poll::Ready(Some(_ev)) => handler(PressEvent {}),
                _ => (),
            }
        });
        let _ = this.on_press_out.as_mut().map(|(listener, handler)| {
            match Pin::new(listener).poll_next(cx) {
                Poll::Ready(Some(_ev)) => handler(PressEvent {}),
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
        let button = gtk::Button::new();
        let on_press = (!is_dummy(self.on_press)).then(|| {
            let listener = EventHandler::new();
            let receiver = listener.get_receiver();
            button.connect_clicked(move |_b| {
                receiver.send(());
            });
            (listener, self.on_press)
        });
        // let on_press_in = (!is_dummy(self.on_press_in)).then(|| {
        //     let listener = EventHandler::new();
        // 	let receiver = listener.get_receiver();
        //     button.connect_clicked(move |_b| {
        // 		receiver.send(());
        // 	});
        //     (listener, self.on_press_in)
        // });
        // let on_press_out = (!is_dummy(self.on_press_out)).then(|| {
        //     let listener = EventHandler::new();
        // 	let receiver = listener.get_receiver();
        //     button.connect_clicked(move |_b| {
        // 		receiver.send(());
        // 	});
        //     (listener, self.on_press_out)
        // });
        let on_press_in = None;
        let on_press_out = None;

        let future = ButtonFuture {
            children: self.children,
            on_press,
            on_press_in,
            on_press_out,
        };
        ElementFuture::new(
            future,
            WrappedWidget {
                widget: button.upcast(),
                inner_widget: None,
                op: crate::widget::WidgetOp::SingleChild(&ButtonOp),
            },
        )
    }
}
