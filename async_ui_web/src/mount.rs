use std::future::Future;

use async_executor::Task;
use async_ui_web_core::{executor::schedule, window::DOCUMENT};

use crate::executor::get_executor;

#[must_use = "When the returned `Task` is dropped your app unmounts. Call `.detach()` to avoid this."]
pub fn mount_at<F: Future + 'static>(child_future: F, node: web_sys::Node) -> Task<F::Output> {
    let fut = async_ui_web_core::ContainerNodeFuture::new_root(child_future, node);
    let task = get_executor().spawn(fut);
    schedule();
    task
}
pub fn mount<F: Future + 'static>(child_future: F) {
    mount_at(
        child_future,
        DOCUMENT.with(|doc| doc.body().expect("no body").to_owned().into()),
    )
    .detach();
}
