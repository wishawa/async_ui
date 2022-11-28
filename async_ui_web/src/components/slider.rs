use futures_lite::FutureExt;
use observables::{ObservableAs, ObservableAsExt};
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

use crate::DOCUMENT;

use super::{
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
    pub value: Option<&'c dyn ObservableAs<f64>>,
    pub min: Option<&'c dyn ObservableAs<f64>>,
    pub max: Option<&'c dyn ObservableAs<f64>>,
    pub step: Option<&'c dyn ObservableAs<f64>>,
    pub on_change: Option<&'c mut dyn FnMut(SliderChangeEvent)>,
}
pub async fn slider(
    SliderProps {
        value,
        min,
        max,
        step,
        mut on_change,
    }: SliderProps<'_>,
) {
    let elem: HtmlInputElement = DOCUMENT.with(|doc| {
        let elem = doc.create_element("input").expect("create element failed");
        elem.unchecked_into()
    });
    elem.set_type("range");
    let value = value.unwrap_or(&[0.0]);
    let min = min.unwrap_or(&[0.0]);
    let max = max.unwrap_or(&[100.0]);
    let step = step.unwrap_or(&[1.0]);

    let elem_1 = elem.clone();

    let mut handlers = SmallVec::<[_; 1]>::new();
    let manager = EventsManager::new();

    if on_change.is_some() {
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
                        on_change.as_mut().map(|f| f(slider_change_event));
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
