use futures_core::Stream;

use crate::listeners::{Listener, ListenerGroup};

/// A future to wait for some tracked data to change.
/// Use [Tracked::until_change] to get this.
pub struct UntilChange<'a> {
    pub(crate) up: UntilChangeGroup,
    pub(crate) here: UntilChangeGroup,
    pub(crate) down: UntilChangeGroup,
    pub(crate) listener: &'a Listener,
}

/// A part of [UntilChange]. Handles either one of the 3 types of change
/// * inside (`up`) change
/// * here change
/// * down (`down`) change
pub(crate) struct UntilChangeGroup {
    pub(crate) waker_idx: usize,
    pub(crate) version: u64,
}

impl UntilChangeGroup {
    pub(crate) fn new(enabled: bool) -> Self {
        Self {
            version: enabled.then_some(0).unwrap_or(u64::MAX),
            waker_idx: usize::MAX,
        }
    }
    pub(crate) fn poll_is_ready(
        &mut self,
        mut listener: ListenerGroup<'_>,
        waker: &std::task::Waker,
    ) -> bool {
        let mut done = false;
        if self.version == u64::MAX {
            return false;
        }
        match self.version {
            0 => self.version = listener.get_version(),
            last_version if last_version < listener.get_version() => {
                done = true;
                self.version = listener.get_version();
            }
            _ => {}
        }
        let waker_idx = &mut self.waker_idx;
        *waker_idx = listener.add_waker(waker, (*waker_idx != usize::MAX).then_some(*waker_idx));
        done
    }
}

impl<'a> UntilChange<'a> {
    pub(crate) fn new(
        listener: &'a Listener,
        listen_up: bool,
        listen_here: bool,
        listen_down: bool,
    ) -> Self {
        Self {
            up: UntilChangeGroup::new(listen_up),
            here: UntilChangeGroup::new(listen_here),
            down: UntilChangeGroup::new(listen_down),
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
        if this.here.poll_is_ready(this.listener.here(), waker)
            | this.up.poll_is_ready(this.listener.up(), waker)
            | this.down.poll_is_ready(this.listener.down(), waker)
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
