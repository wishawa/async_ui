use scoped_tls::ScopedKey;
use std::future::Future;

use super::control::{vnode::VNode, Control};

pub trait Backend: Sized + 'static {
    type VNode: VNode;
    type Spawner: Spawner;
    fn get_tls() -> &'static ScopedKey<Control<Self>>;
    fn get_dummy_control() -> Control<Self>;
}
pub unsafe trait Spawner: 'static {
    type Task;
    fn spawn<'a, F: Future<Output = ()> + 'static>(future: F) -> Self::Task;
}
