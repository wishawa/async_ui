use std::future::pending;

use futures_lite::FutureExt;
use glib::Cast;
use gtk::traits::ButtonExt;

use crate::{
    widget::{single::ButtonOp, WidgetOp, WrappedWidget},
    Fragment,
};

use super::{
    events::{EventsManager, QueuedEvent},
    ElementFuture,
};

pub struct PressEvent {}
#[derive(Default)]
pub struct ButtonProps<'c> {
    pub children: Option<Fragment<'c>>,
    pub on_press: Option<&'c mut (dyn FnMut(PressEvent) + 'c)>,
}

pub async fn button<'c>(
    ButtonProps {
        children,
        mut on_press,
    }: ButtonProps<'c>,
) {
    let button = gtk::Button::new();
    let manager = EventsManager::new();
    if on_press.is_some() {
        let mgr = manager.clone();
        button.connect_clicked(move |_b| mgr.add_event(QueuedEvent::Click));
    }
    ElementFuture::new(
        (async {
            if let Some(children) = children {
                children.await;
            } else {
                pending::<()>().await;
            }
        })
        .or(async {
            manager.grab_waker().await;
            loop {
                let mut events = manager.get_queue().await;
                for event in events.drain(..) {
                    match event {
                        QueuedEvent::Click => {
                            on_press.as_mut().map(|f| f(PressEvent {}));
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

// pin_project! {
//     pub struct ButtonFuture<'c> {
//         #[pin] children: Fragment<'c>,
//         on_press: &'c mut (dyn FnMut(PressEvent) + 'c),
//         manager: Rc<EventsManager>
//     }
// }
// impl<'c> Future for ButtonFuture<'c> {
//     type Output = ();

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let this = self.project();
//         if let Some(mut events) = this.manager.borrow_queue_mut() {
//             for event in events.drain(..) {
//                 match event {
//                     QueuedEvent::Click => (this.on_press)(PressEvent {}),
//                     _ => {}
//                 }
//             }
//         }
//         this.children.poll(cx)
//     }
// }
// impl<'c> IntoFuture for Button<'c> {
//     type Output = ();
//     type IntoFuture = ElementFuture<ButtonFuture<'c>>;

//     fn into_future(self) -> Self::IntoFuture {
//         let button = gtk::Button::new();
//         let Self { children, on_press } = self;
//         let manager = EventsManager::new();
//         if !is_dummy(on_press) {
//             let mgr = manager.clone();
//             button.connect_clicked(move |_b| mgr.add_event(QueuedEvent::Click));
//         }
//         let future = ButtonFuture {
//             children,
//             on_press,
//             manager,
//         };
//         ElementFuture::new(
//             future,
//             WrappedWidget {
//                 widget: button.upcast(),
//                 inner_widget: None,
//                 op: crate::widget::WidgetOp::SingleChild(&ButtonOp),
//             },
//         )
//     }
// }
