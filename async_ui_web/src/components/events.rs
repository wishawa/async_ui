use std::{
    cell::{RefCell, RefMut},
    collections::VecDeque,
    ops::AddAssign,
    rc::Rc,
    task::Waker,
};

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
}

pub(super) struct EventsManager {
    inner: RefCell<EventHandlerInner>,
}
struct EventHandlerInner {
    events: VecDeque<QueuedEvent>,
    waker: Option<Waker>,
    version: u64,
    last_version: u64,
}

impl EventsManager {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            inner: RefCell::new(EventHandlerInner {
                events: VecDeque::new(),
                waker: None,
                version: 1,
                last_version: 1,
            }),
        })
    }
    pub fn add_event(&self, event: QueuedEvent) {
        let bm = &mut *self.inner.borrow_mut();
        bm.events.push_back(event);
        bm.version.add_assign(1);
        if let Some(waker) = bm.waker.as_ref() {
            waker.wake_by_ref();
        }
    }
    pub fn borrow_queue_mut<'b>(&'b self) -> Option<RefMut<'b, VecDeque<QueuedEvent>>> {
        let mut bm = self.inner.borrow_mut();
        if bm.last_version < bm.version {
            bm.last_version = bm.version;
            Some(RefMut::map(bm, |v| &mut v.events))
        } else {
            None
        }
    }
    pub fn set_waker(&self, waker: &Waker) {
        let mut bm = self.inner.borrow_mut();
        if bm.waker.is_none() {
            bm.waker = Some(waker.clone());
        }
    }
}

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use crate::executor::schedule;

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
            schedule();
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
