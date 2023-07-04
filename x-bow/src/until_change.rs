use async_ui_internal_utils::wakers_list::{WakerSlot, WakersList};
use futures_core::Stream;

use crate::listeners::{Listener, ListenerGroup};

/// A future to wait for some tracked data to change.
/// Use [Tracked::until_change] to get this.
pub struct UntilChange<'a> {
    pub(crate) up_here_down: [UntilChangeGroup; 3],
    pub(crate) listener: &'a Listener<'a>,
}

/// A part of [UntilChange]. Handles either one of the 3 types of change
/// * inside (`up`) change
/// * here change
/// * down (`down`) change
pub(crate) struct UntilChangeGroup {
    pub(crate) version: u64,
    pub(crate) waker_slot: WakerSlot,
}

impl UntilChangeGroup {
    pub(crate) fn new(input: Option<(&mut WakersList, &ListenerGroup)>) -> Self {
        if let Some((list, group)) = input {
            Self {
                version: 0,
                waker_slot: list.add(&group.list),
            }
        } else {
            Self {
                version: u64::MAX,
                waker_slot: WakerSlot::INVALID,
            }
        }
    }
    pub(crate) fn poll_is_ready(
        &mut self,
        waker: &std::task::Waker,
        full_list: &mut WakersList,
        group: &ListenerGroup,
    ) -> bool {
        let mut done = false;
        if self.version == u64::MAX {
            return false;
        }
        let new_version = group.get_version();
        match self.version {
            0 => self.version = new_version,
            last_version if last_version < new_version => {
                done = true;
                self.version = new_version;
            }
            _ => {}
        }
        full_list.update(&self.waker_slot, waker);
        done
    }
    pub(crate) fn unlisten(&mut self, list: &mut WakersList) {
        if self.version != u64::MAX {
            list.remove(&self.waker_slot);
        }
    }
}

impl<'a> UntilChange<'a> {
    pub(crate) fn new(
        listener: &'a Listener,
        listen_up: bool,
        listen_here: bool,
        listen_down: bool,
    ) -> Self {
        let enabled: [bool; 3] = [listen_down, listen_here, listen_up];
        let mut full = listener.full_list.borrow_mut();
        Self {
            up_here_down: std::array::from_fn(|idx| {
                UntilChangeGroup::new(
                    enabled[idx].then(|| (&mut *full, &listener.up_here_down[idx])),
                )
            }),
            listener,
        }
    }
}

impl<'a> futures_core::Stream for UntilChange<'a> {
    type Item = ();

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let waker = cx.waker();
        let mut full_list = this.listener.full_list.borrow_mut();
        if this
            .up_here_down
            .iter_mut()
            .zip(this.listener.up_here_down.iter())
            .fold(false, |prev, (uc, group)| {
                prev | uc.poll_is_ready(waker, &mut *full_list, group)
            })
        {
            std::task::Poll::Ready(Some(()))
        } else {
            std::task::Poll::Pending
        }
    }
}

impl<'a> std::future::Future for UntilChange<'a> {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.poll_next(cx).map(|_| ())
    }
}

impl<'a> Drop for UntilChange<'a> {
    fn drop(&mut self) {
        let mut list = self.listener.full_list.borrow_mut();
        self.up_here_down.iter_mut().for_each(|uc| {
            uc.unlisten(&mut *list);
        })
    }
}
