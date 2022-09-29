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

pub async fn button<'c, I: IntoIterator<Item = ButtonProp<'c>>>(props: I) {
    button_inner(&mut props.into_iter()).await;
}
async fn button_inner<'c>(props: &mut dyn Iterator<Item = ButtonProp<'c>>) {
    let button = DOCUMENT.with(|doc| {
        let elem = doc.create_element("button").expect("create element failed");
        let elem: HtmlButtonElement = elem.unchecked_into();
        elem
    });

    let mut handlers = SmallVec::<[_; 1]>::new();
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
