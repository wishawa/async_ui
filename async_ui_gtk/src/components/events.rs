use std::{
    cell::{RefCell, RefMut},
    collections::VecDeque,
    ops::AddAssign,
    rc::Rc,
    task::Waker,
};

pub(super) enum QueuedEvent {
    Click,
    MouseDown,
    MouseUp,
    Input,
    Submit,
    KeyPress,
    KeyUp,
    KeyDown,
    Focus,
    Blur,
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
}
