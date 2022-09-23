use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use glib::Cast;
use gtk::traits::ButtonExt;
use pin_project_lite::pin_project;

use crate::{
    widget::{single::ButtonOp, WrappedWidget},
    Fragment,
};

use super::{
    dummy::{create_dummy, is_dummy},
    events::{EventsManager, QueuedEvent},
    ElementFuture,
};

pub struct PressEvent {}
pub struct Button<'c> {
    pub children: Fragment<'c>,
    pub on_press: &'c mut (dyn FnMut(PressEvent) + 'c),
}

impl<'c> Default for Button<'c> {
    fn default() -> Self {
        Self {
            children: Default::default(),
            on_press: create_dummy(),
        }
    }
}

pin_project! {
    pub struct ButtonFuture<'c> {
        #[pin] children: Fragment<'c>,
        on_press: &'c mut (dyn FnMut(PressEvent) + 'c),
        manager: Rc<EventsManager>
    }
}
impl<'c> Future for ButtonFuture<'c> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Some(mut events) = this.manager.borrow_queue_mut() {
            for event in events.drain(..) {
                match event {
                    QueuedEvent::Click => (this.on_press)(PressEvent {}),
                    _ => {}
                }
            }
        }
        this.children.poll(cx)
    }
}
impl<'c> IntoFuture for Button<'c> {
    type Output = ();
    type IntoFuture = ElementFuture<ButtonFuture<'c>>;

    fn into_future(self) -> Self::IntoFuture {
        let button = gtk::Button::new();
        let Self { children, on_press } = self;
        let manager = EventsManager::new();
        if !is_dummy(on_press) {
            let mgr = manager.clone();
            button.connect_clicked(move |_b| mgr.add_event(QueuedEvent::Click));
        }
        let future = ButtonFuture {
            children,
            on_press,
            manager,
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
