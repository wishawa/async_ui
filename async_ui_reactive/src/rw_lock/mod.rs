pub mod reactive;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct IMCell<T>(RwLock<T>);
type LockReadGuard<'l, T> = RwLockReadGuard<'l, T>;
type LockWriteGuard<'l, T> = RwLockWriteGuard<'l, T>;
impl<T> IMCell<T> {
    fn lock_read(&self) -> LockReadGuard<'_, T> {
        self.0.read().expect("poisoned")
    }
    fn lock_write(&self) -> LockWriteGuard<'_, T> {
        self.0.write().expect("poisoned")
    }
    fn new(value: T) -> Self {
        Self(RwLock::new(value))
    }
}
