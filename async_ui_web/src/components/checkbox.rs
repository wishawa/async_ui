use futures_lite::FutureExt;
use observables::{ObservableAs, ObservableAsExt};
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

use crate::{utils::class_list::ClassList, window::DOCUMENT};

use super::{
    dummy::{dummy_handler, is_dummy_handler},
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

pub struct CheckboxProps<'c> {
    pub value: &'c dyn ObservableAs<bool>,
    pub on_change: &'c mut dyn FnMut(CheckboxChangeEvent),
    pub class: Option<&'c ClassList<'c>>,
}
impl<'c> Default for CheckboxProps<'c> {
    fn default() -> Self {
        Self {
            value: &[false],
            on_change: dummy_handler(),
            class: None,
        }
    }
}

/** Checkbox - HTML <input type="checkbox">
 *
 */
pub async fn checkbox<'c>(
    CheckboxProps {
        value,
        on_change,
        class,
    }: CheckboxProps<'c>,
) {
    let elem: HtmlInputElement = DOCUMENT.with(|doc| {
        let elem = doc.create_element("input").expect("create element failed");
        elem.unchecked_into()
    });
    elem.set_type("checkbox");
    let mut handlers = SmallVec::<[_; 1]>::new();
    let manager = EventsManager::new();
    if !is_dummy_handler(on_change) {
        let h = create_handler(&manager, |_ev: Event| QueuedEvent::Change());
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
                    QueuedEvent::Change() => {
                        on_change(checkbox_change_event);
                    }
                    _ => {}
                }
            }
        }
    })
    .or(value.for_each(|v| elem_2.set_checked(*v)));
    ElementFuture::new(future, elem.into()).await;
}
