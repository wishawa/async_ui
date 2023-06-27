use std::future::Future;

use async_executor::Task;
use async_ui_web_core::{executor::schedule, window::DOCUMENT};
use wasm_bindgen::UnwrapThrowExt;

use crate::executor::get_executor;

/// Start running the given future, letting it render into the given node.
///
/// ```
/// # use async_ui_web::mount_at;
/// # let my_app = || std::future::pending::<()>();
/// # let _ = || {
/// let mounted_task = mount_at(my_app(), web_sys::window().unwrap().document().unwrap().into());
/// mounted_task.detach();
/// # };
/// ```
///
/// The return value is a [Task]. When dropped, it will unmount your app.
/// To prevent unmounting, call [detach][Task::detach] first.
#[must_use = "When the returned `Task` is dropped your app unmounts. Call `.detach()` to avoid this."]
pub fn mount_at<F: Future + 'static>(child_future: F, node: web_sys::Node) -> Task<F::Output> {
    let fut = async_ui_web_core::ContainerNodeFuture::new_root(child_future, node);
    let task = get_executor().spawn(fut);
    schedule();
    task
}
/// Start running the given future, letting it render into the `<body>` of the document.
///
/// ```
/// # use async_ui_web::mount;
/// # let my_app = || std::future::pending::<()>();
/// # let _ = || {
/// mount(my_app());
/// # };
/// ```
///
/// The [mount_at] function provides more options, if you need.
pub fn mount<F: Future + 'static>(child_future: F) {
    mount_at(
        child_future,
        DOCUMENT.with(|doc| doc.body().unwrap_throw().into()),
    )
    .detach();
}
