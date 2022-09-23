use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, MouseEvent};

use crate::{window::DOCUMENT, Fragment};

use super::{
    dummy::create_dummy,
    events::{maybe_create_handler, EventHandler, EventsManager, QueuedEvent},
    ElementFuture,
};

pub struct PressEvent {
    pub native_event: MouseEvent,
}
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
        handlers: SmallVec<[EventHandler<'c>; 3]>,
        manager: Rc<EventsManager>
    }
}
impl<'c> Future for ButtonFuture<'c> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Some(mut events) = this.manager.borrow_queue_mut() {
            for ev in events.drain(..) {
                match ev {
                    QueuedEvent::Click(native_event) => {
                        (this.on_press)(PressEvent { native_event })
                    }
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
        let button = DOCUMENT.with(|doc| {
            let elem = doc.create_element("button").expect("create element failed");
            let elem: HtmlButtonElement = elem.unchecked_into();
            elem
        });
        let Self { children, on_press } = self;
        let mut handlers = SmallVec::new();
        let manager = EventsManager::new();
        if let Some(h) = maybe_create_handler(&manager, on_press, |e| QueuedEvent::Click(e)) {
            button.set_onclick(Some(h.get_function()));
            handlers.push(h);
        }
        let future = ButtonFuture {
            children,
            on_press,
            manager,
            handlers,
        };
        ElementFuture::new(future, button.into())
    }
}
