use std::sync::{Arc, Mutex, MutexGuard, Weak};

pub mod backend;
pub mod control;
pub mod drop_check;
pub mod element;
pub mod render;
pub mod wrappers;

pub trait MaybeSend: Send {}
impl<T: Send> MaybeSend for T {}
pub trait MaybeSync: Sync {}
impl<T: Sync> MaybeSync for T {}

type Shared<T> = Arc<T>;
type SharedWeak<T> = Weak<T>;
type Mutable<T> = Mutex<T>;
fn mutable_borrow_mut<T>(m: &Mutable<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap()
}
