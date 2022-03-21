use std::collections::VecDeque;

use super::{reactive::Reactive, Shared};

type ChannelInner<T> = Reactive<VecDeque<T>>;
pub struct ChannelEntry<T> {
    inner: Shared<ChannelInner<T>>,
}
pub struct ChannelExit<T> {
    inner: Shared<ChannelInner<T>>,
}
impl<T> ChannelEntry<T> {
    pub fn send(&self, value: T) {
        let mut bm = self.inner.borrow_mut();
        bm.push_back(value);
    }
}
impl<T> ChannelExit<T> {
    pub fn receive_now(&self) -> Vec<T> {
        let mut bm = self.inner.borrow_mut();
        bm.drain(..).collect()
    }
    pub async fn receive(&self) -> T {
        loop {
            let mut bm = self.inner.borrow_next_mut().await;
            if let Some(v) = bm.pop_front() {
                break v;
            }
        }
    }
}
pub fn create_channel<T>() -> (ChannelEntry<T>, ChannelExit<T>) {
    let inner = Shared::new(ChannelInner::new(VecDeque::new()));
    (
        ChannelEntry {
            inner: inner.clone(),
        },
        ChannelExit { inner },
    )
}
