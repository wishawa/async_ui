use std::future::Future;

use glib::MainContext;
pub(crate) fn set_executor_future<F: Future<Output = ()> + 'static>(future: F) {
    MainContext::default().spawn_local(future);
}
