use std::future::pending;

use futures_lite::FutureExt;
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, MouseEvent};

use crate::{utils::class_list::ClassList, window::DOCUMENT, Fragment};

use super::{
    events::{create_handler, EventsManager, QueuedEvent},
    ElementFuture,
};

#[derive(Default)]
pub struct ButtonProps<'c> {
    pub children: Option<Fragment<'c>>,
    pub on_press: Option<&'c mut dyn FnMut(PressEvent)>,
    pub class: Option<&'c ClassList<'c>>,
}

pub struct PressEvent {
    pub native_event: MouseEvent,
}

pub async fn button<'c>(
    ButtonProps {
        children,
        mut on_press,
        class,
    }: ButtonProps<'c>,
) {
    let button = DOCUMENT.with(|doc| {
        let elem = doc.create_element("button").expect("create element failed");
        let elem: HtmlButtonElement = elem.unchecked_into();
        elem
    });

    let mut handlers = SmallVec::<[_; 1]>::new();
    let manager = EventsManager::new();

    if on_press.is_some() {
        let h = create_handler(&manager, |e| QueuedEvent::Click(e));
        button.set_onclick(Some(h.get_function()));
        handlers.push(h);
    }
    if let Some(class) = class {
        class.set_dom(button.class_list());
    }

    let future = (async {
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
                    QueuedEvent::Click(native_event) => {
                        on_press.as_mut().map(|f| f(PressEvent { native_event }));
                    }
                    _ => {}
                }
            }
        }
    });
    ElementFuture::new(future, button.into()).await
}
