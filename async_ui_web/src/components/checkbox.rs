use futures_lite::FutureExt;
use observables::{ObservableAs, ObservableAsExt};
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

use crate::{utils::class_list::ClassList, window::DOCUMENT};

use super::{
    events::{create_handler, EventsManager, QueuedEvent},
    ElementFuture,
};

pub struct CheckboxChangeEvent {
    node: HtmlInputElement,
}
impl CheckboxChangeEvent {
    pub fn get_value(&self) -> bool {
        self.node.checked()
    }
}

#[derive(Default)]
pub struct CheckboxProps<'c> {
    pub value: Option<&'c dyn ObservableAs<bool>>,
    pub on_change: Option<&'c mut dyn FnMut(CheckboxChangeEvent)>,
    pub class: Option<&'c ClassList<'c>>,
}

pub async fn checkbox<'c>(
    CheckboxProps {
        value,
        mut on_change,
        class,
    }: CheckboxProps<'c>,
) {
    let elem: HtmlInputElement = DOCUMENT.with(|doc| {
        let elem = doc.create_element("input").expect("create element failed");
        elem.unchecked_into()
    });
    elem.set_type("checkbox");
    let value = value.unwrap_or(&[false]);
    let mut handlers = SmallVec::<[_; 1]>::new();
    let manager = EventsManager::new();
    if on_change.is_some() {
        let h = create_handler(&manager, |_ev: Event| QueuedEvent::Check());
        elem.set_onchange(Some(h.get_function()));
        handlers.push(h);
    }
    if let Some(cl) = class {
        cl.set_dom(elem.class_list());
    }
    let elem_1 = elem.clone();
    let elem_2 = elem.clone();
    let future = (async {
        loop {
            let mut events = manager.get_queue().await;
            for event in events.drain(..) {
                let checkbox_change_event = CheckboxChangeEvent {
                    node: elem_1.clone(),
                };
                match event {
                    QueuedEvent::Check() => {
                        on_change.as_mut().map(|f| f(checkbox_change_event));
                    }
                    _ => {}
                }
            }
        }
    })
    .or(value.for_each(|v| elem_2.set_checked(*v)));
    ElementFuture::new(future, elem.into()).await;
}
