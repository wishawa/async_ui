use std::{future::Future, marker::PhantomData, pin::Pin};

use scoped_tls::ScopedKey;

use crate::control::{vnode::VNode, Control};

pub trait Backend: Sized + 'static {
    type VNode: VNode;
    type Spawner: Spawner;
    fn get_tls() -> &'static ScopedKey<Control<Self>>;
    fn get_dummy_control() -> Control<Self>;
}
pub unsafe trait Spawner: 'static {
    type Task;
    fn spawn<'a, F: Future<Output = ()> + 'static>(future: F) -> Self::Task;
    fn wake_now();
    fn schedule_now();
}
