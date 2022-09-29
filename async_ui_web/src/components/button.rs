use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, MouseEvent};

use crate::{utils::class_list::ClassList, window::DOCUMENT, Fragment};

use super::{
    events::{create_handler, EventHandler, EventsManager, QueuedEvent},
    ElementFuture,
};

#[derive(Default)]
pub enum ButtonProp<'c> {
    Children(Fragment<'c>),
    OnPress(&'c mut dyn FnMut(PressEvent)),
    Class(&'c ClassList<'c>),
    #[default]
    Null,
}

pub struct PressEvent {
    pub native_event: MouseEvent,
}

pin_project! {
    pub struct ButtonFuture<'c> {
        #[pin] children: Fragment<'c>,
        on_press: Option<&'c mut (dyn FnMut(PressEvent) + 'c)>,
        handlers: SmallVec<[EventHandler<'c>; 3]>,
        manager: Rc<EventsManager>,
        first: bool
    }
}
impl<'c> Future for ButtonFuture<'c> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if *this.first {
            this.manager.set_waker(cx.waker());
            *this.first = false;
        }
        if let Some(mut events) = this.manager.borrow_queue_mut() {
            for ev in events.drain(..) {
                match ev {
                    QueuedEvent::Click(native_event) => {
                        this.on_press
                            .as_mut()
                            .map(|f| f(PressEvent { native_event }));
                    }
                    _ => {}
                }
            }
        }
        this.children.poll(cx)
    }
}
pub async fn button<'c, I: IntoIterator<Item = ButtonProp<'c>>>(props: I) {
    let button = DOCUMENT.with(|doc| {
        let elem = doc.create_element("button").expect("create element failed");
        let elem: HtmlButtonElement = elem.unchecked_into();
        elem
    });

    let mut handlers = SmallVec::new();
    let manager = EventsManager::new();

    let mut children = None;
    let mut on_press = None;
    for prop in props {
        match prop {
            ButtonProp::Children(v) => children = Some(v),
            ButtonProp::OnPress(v) => {
                let h = create_handler(&manager, |e| QueuedEvent::Click(e));
                button.set_onclick(Some(h.get_function()));
                handlers.push(h);
                on_press = Some(v);
            }
            ButtonProp::Class(v) => {
                v.set_dom(button.class_list());
            }
            ButtonProp::Null => {}
        }
    }

    let future = ButtonFuture {
        children: children.unwrap_or_default(),
        on_press,
        manager,
        handlers,
        first: true,
    };
    ElementFuture::new(future, button.into()).await
}
