use std::{
    cell::{RefCell, RefMut},
    collections::VecDeque,
    future::poll_fn,
    ops::AddAssign,
    rc::Rc,
    task::{Poll, Waker},
};

pub struct EventsManager<E> {
    inner: RefCell<EventHandlerInner<E>>,
}
struct EventHandlerInner<E> {
    events: VecDeque<E>,
    waker: Option<Waker>,
    version: u64,
    last_version: u64,
}

impl<E> EventsManager<E> {
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
    pub fn add_event(&self, event: E) {
        let bm = &mut *self.inner.borrow_mut();
        bm.events.push_back(event);
        bm.version.add_assign(1);
        if let Some(waker) = bm.waker.as_ref() {
            waker.wake_by_ref();
        }
    }
    fn borrow_queue_mut<'b>(&'b self) -> Option<RefMut<'b, VecDeque<E>>> {
        let mut bm = self.inner.borrow_mut();
        if bm.last_version < bm.version {
            bm.last_version = bm.version;
            Some(RefMut::map(bm, |v| &mut v.events))
        } else {
            None
        }
    }
    fn set_waker(&self, waker: Waker) {
        let mut bm = self.inner.borrow_mut();
        if bm.waker.is_none() {
            bm.waker = Some(waker.clone());
        }
    }
    pub async fn grab_waker(&self) {
        let waker = poll_fn(|cx| Poll::Ready(cx.waker().clone())).await;
        self.set_waker(waker);
    }
    pub async fn get_queue<'b>(&'b self) -> RefMut<'b, VecDeque<E>> {
        poll_fn(|_cx| match self.borrow_queue_mut() {
            Some(q) => Poll::Ready(q),
            None => Poll::Pending,
        })
        .await
    }
}
