use futures_lite::FutureExt;
use glib::Cast;
use gtk::traits::ButtonExt;

use crate::{
    widget::{single::ButtonOp, WidgetOp, WrappedWidget},
    Fragment,
};

use super::{
    dummy::{dummy_handler, is_dummy_handler},
    events::{EventsManager, QueuedEvent},
    ElementFuture,
};

pub struct PressEvent {}
pub struct ButtonProps<'c> {
    pub children: Fragment<'c>,
    pub on_press: &'c mut (dyn FnMut(PressEvent) + 'c),
}
impl<'c> Default for ButtonProps<'c> {
    fn default() -> Self {
        Self {
            children: Default::default(),
            on_press: dummy_handler(),
        }
    }
}

pub async fn button<'c>(ButtonProps { children, on_press }: ButtonProps<'c>) {
    let button = gtk::Button::new();
    let manager = EventsManager::new();
    if !is_dummy_handler(on_press) {
        let mgr = manager.clone();
        button.connect_clicked(move |_b| mgr.add_event(QueuedEvent::Click));
    }
    ElementFuture::new(
        (children).or(async {
            manager.grab_waker().await;
            loop {
                let mut events = manager.get_queue().await;
                for event in events.drain(..) {
                    match event {
                        QueuedEvent::Click => {
                            on_press(PressEvent {});
                        }
                        _ => {}
                    }
                }
            }
        }),
        WrappedWidget {
            widget: button.clone().upcast(),
            inner_widget: button.upcast(),
            op: WidgetOp::SingleChild(&ButtonOp),
        },
    )
    .await;
}
