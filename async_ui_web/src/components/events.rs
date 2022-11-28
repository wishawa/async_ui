use std::rc::Rc;

use js_sys::Function;
use web_sys::{Event, FocusEvent, InputEvent, KeyboardEvent, MouseEvent};

pub(super) enum QueuedEvent {
    Click(MouseEvent),
    // MouseDown(MouseEvent),
    // MouseUp(MouseEvent),
    Input(InputEvent),
    KeyPress(KeyboardEvent),
    // KeyUp(KeyboardEvent),
    // KeyDown(KeyboardEvent),
    Focus(FocusEvent),
    Blur(FocusEvent),
    Change(),
}

pub(super) type EventsManager = async_ui_props::events::EventsManager<QueuedEvent>;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use crate::executor::run_now;

pub(super) struct EventHandler<'h> {
    closure: Closure<dyn Fn(Event) + 'h>,
}

impl<'h> EventHandler<'h> {
    pub fn new<E: wasm_bindgen::convert::FromWasmAbi + JsCast + 'static, F: Fn(E) + 'static>(
        execute: F,
    ) -> Self {
        let closure = Closure::new(move |event: Event| {
            let event: E = event.unchecked_into();
            execute(event);
            run_now();
        });
        Self { closure }
    }
    pub fn get_function(&self) -> &Function {
        self.closure.as_ref().unchecked_ref()
    }
}

pub(super) fn create_handler<
    'h,
    E: wasm_bindgen::convert::FromWasmAbi + JsCast + 'static,
    M: (Fn(E) -> QueuedEvent) + 'static,
>(
    manager: &Rc<EventsManager>,
    map_ev: M,
) -> EventHandler<'h> {
    let manager = manager.clone();
    EventHandler::new(move |event: E| {
        let q = map_ev(event);
        manager.add_event(q);
    })
}
