use futures_lite::FutureExt;
use observables::{ObservableAs, ObservableAsExt};
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

use crate::DOCUMENT;

use super::{
    dummy::{dummy_handler, is_dummy_handler},
    events::{create_handler, EventsManager, QueuedEvent},
    ElementFuture,
};

pub struct SliderChangeEvent {
    node: HtmlInputElement,
}

impl SliderChangeEvent {
    pub fn get_value(&self) -> f64 {
        self.node.value_as_number()
    }
}

pub struct SliderProps<'c> {
    pub value: &'c dyn ObservableAs<f64>,
    pub min: &'c dyn ObservableAs<f64>,
    pub max: &'c dyn ObservableAs<f64>,
    pub step: &'c dyn ObservableAs<f64>,
    pub on_change: &'c mut dyn FnMut(SliderChangeEvent),
}

impl<'c> Default for SliderProps<'c> {
    fn default() -> Self {
        Self {
            value: &[0.0],
            min: &[0.0],
            max: &[100.0],
            step: &[1.0],
            on_change: dummy_handler(),
        }
    }
}
/** Slider - HTML <input type="range">
 *
 */
pub async fn slider(
    SliderProps {
        value,
        min,
        max,
        step,
        on_change,
    }: SliderProps<'_>,
) {
    let elem: HtmlInputElement = DOCUMENT.with(|doc| {
        let elem = doc.create_element("input").expect("create element failed");
        elem.unchecked_into()
    });
    elem.set_type("range");

    let elem_1 = elem.clone();

    let mut handlers = SmallVec::<[_; 1]>::new();
    let manager = EventsManager::new();

    if !is_dummy_handler(on_change) {
        let h = create_handler(&manager, |_ev: Event| QueuedEvent::Change());
        elem.set_onchange(Some(h.get_function()));
        handlers.push(h);
    }

    let future = (async {
        loop {
            let mut events = manager.get_queue().await;
            for event in events.drain(..) {
                let slider_change_event = SliderChangeEvent {
                    node: elem_1.clone(),
                };
                match event {
                    QueuedEvent::Change() => {
                        on_change(slider_change_event);
                    }
                    _ => {}
                }
            }
        }
    })
    .or(value.for_each(|v| elem_1.set_value_as_number(*v)))
    .or(min.for_each(|v| elem_1.set_min(&v.to_string())))
    .or(max.for_each(|v| elem_1.set_max(&v.to_string())))
    .or(step.for_each(|v| elem_1.set_step(&v.to_string())));

    ElementFuture::new(future, elem.into()).await;
}
