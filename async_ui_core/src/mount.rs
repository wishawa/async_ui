use std::future::Future;

use scoped_async_spawn::GiveUnforgettableScope;

use crate::{
    backend::BackendTrait,
    executor::{get_driving_future, spawn_local},
};

pub fn mount<B: BackendTrait, F: Future<Output = ()> + 'static>(fut: F) {
    B::drive_executor(get_driving_future());
    spawn_local(GiveUnforgettableScope::new_static(fut)).detach();
}
