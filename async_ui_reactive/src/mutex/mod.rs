pub mod channel;
pub mod reactive;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug)]
struct IMCell<T>(Mutex<T>);
type Shared<T> = Arc<T>;
type LockReadGuard<'l, T> = MutexGuard<'l, T>;
type LockWriteGuard<'l, T> = MutexGuard<'l, T>;
impl<T> IMCell<T> {
    fn lock_read(&self) -> LockReadGuard<'_, T> {
        self.0.lock().expect("poisoned")
    }
    fn lock_write(&self) -> LockWriteGuard<'_, T> {
        self.0.lock().expect("poisoned")
    }
    fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }
}
