use scoped_tls::ScopedKey;
use std::future::Future;

use super::{control::Control, MaybeSend};

pub trait Backend: Sized + 'static {
    type Spawner: Spawner;
    type NodeType: MaybeSend + Clone + 'static;
    fn get_tls() -> &'static ScopedKey<Control<Self>>;
    fn get_dummy_control() -> Control<Self>;
}
pub unsafe trait Spawner: 'static {
    type Task: MaybeSend;
    fn spawn<'a, F: Future<Output = ()> + 'static>(future: F) -> Self::Task;
}
