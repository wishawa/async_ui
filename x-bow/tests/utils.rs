use core::{pin::Pin, task::Context};

use async_ui_internal_utils::dummy_waker::dummy_waker;
use futures_core::Stream;

pub fn is_ready(s: Pin<&mut dyn Stream<Item = ()>>) -> bool {
    let waker = dummy_waker();
    let mut ctx = Context::from_waker(&waker);
    s.poll_next(&mut ctx).is_ready()
}

pub fn is_all_pending<const N: usize>(s: [Pin<&mut dyn Stream<Item = ()>>; N]) -> bool {
    s.into_iter().all(|one| !is_ready(one))
}

pub fn is_all_ready<const N: usize>(s: [Pin<&mut dyn Stream<Item = ()>>; N]) -> bool {
    s.into_iter().all(|one| is_ready(one))
}
