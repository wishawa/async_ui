use futures_lite::FutureExt;
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, MouseEvent};

use crate::{utils::class_list::ClassList, window::DOCUMENT, Fragment};

use super::{
    dummy::{dummy_handler, is_dummy_handler},
    events::{create_handler, EventsManager, QueuedEvent},
    ElementFuture,
};

pub struct ButtonProps<'c> {
    pub children: Fragment<'c>,
    pub on_press: &'c mut dyn FnMut(PressEvent),
    pub class: Option<&'c ClassList<'c>>,
}
impl<'c> Default for ButtonProps<'c> {
    fn default() -> Self {
        Self {
            children: Default::default(),
            on_press: dummy_handler(),
            class: None,
        }
    }
}

pub struct PressEvent {
    pub native_event: MouseEvent,
}

pub async fn button<'c>(
    ButtonProps {
        children,
        on_press,
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

    if !is_dummy_handler(on_press) {
        let h = create_handler(&manager, |e| QueuedEvent::Click(e));
        button.set_onclick(Some(h.get_function()));
        handlers.push(h);
    }
    if let Some(class) = class {
        class.set_dom(button.class_list());
    }

    let future = children.or(async {
        manager.grab_waker().await;
        loop {
            let mut events = manager.get_queue().await;
            for event in events.drain(..) {
                match event {
                    QueuedEvent::Click(native_event) => {
                        on_press(PressEvent { native_event });
                    }
                    _ => {}
                }
            }
        }
    });
    ElementFuture::new(future, button.into()).await
}
