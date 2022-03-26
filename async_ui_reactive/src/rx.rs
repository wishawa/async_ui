use std::{
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError,
    },
};

use super::subscriptions::Subscriptions;

pub struct Rx<T> {
    data: RwLock<T>,
    subscriptions: Mutex<Subscriptions>,
    version: AtomicUsize,
}

impl<T> Rx<T> {
    pub fn new(value: T) -> Self {
        Self {
            data: RwLock::new(value),
            subscriptions: Mutex::new(Subscriptions::new()),
            version: AtomicUsize::new(0),
        }
    }
    pub fn try_borrow<'a>(&'a self) -> Result<RxGuard<'a, T>, TryLockError<RwLockReadGuard<T>>> {
        let guard = self.data.try_read()?;
        Ok(RxGuard { guard })
    }
    fn try_borrow_mut_base<'a, const SILENT: bool>(
        &'a self,
    ) -> Result<RxGuardMutBase<'a, T, SILENT>, TryLockError<RwLockWriteGuard<T>>> {
        let guard = self.data.try_write()?;
        Ok(RxGuardMutBase { guard, rx: self })
    }
    pub fn try_borrow_mut<'a>(
        &'a self,
    ) -> Result<RxGuardMut<'a, T>, TryLockError<RwLockWriteGuard<T>>> {
        self.try_borrow_mut_base()
    }
    pub fn try_borrow_mut_silent<'a>(
        &'a self,
    ) -> Result<RxGuardMutSilent<'a, T>, TryLockError<RwLockWriteGuard<T>>> {
        self.try_borrow_mut_base()
    }
    pub fn borrow<'a>(&'a self) -> RxGuard<'a, T> {
        self.try_borrow().expect("Rx borrow failed")
    }
    pub fn borrow_mut<'a>(&'a self) -> RxGuardMut<'a, T> {
        self.try_borrow_mut().expect("Rx borrow_mut failed")
    }
    pub fn borrow_mut_silent<'a>(&'a self) -> RxGuardMutSilent<'a, T> {
        self.try_borrow_mut_silent()
            .expect("Rx borrow_mut_silent failed")
    }
    pub fn visit<R, F: FnOnce(&T) -> R>(&self, func: F) -> R {
        let b = self.borrow();
        func(&*b)
    }
    pub fn visit_mut<R, F: FnOnce(&mut T) -> R>(&self, func: F) -> R {
        let mut b = self.borrow_mut();
        func(&mut *b)
    }
    pub fn visit_mut_silent<R, F: FnOnce(&mut T) -> R>(&self, func: F) -> R {
        let mut b = self.borrow_mut_silent();
        func(&mut *b)
    }
    pub fn replace(&self, new_value: T) -> T {
        self.visit_mut(|rm| std::mem::replace(rm, new_value))
    }
    pub fn replace_silent(&self, new_value: T) -> T {
        self.visit_mut_silent(|rm| std::mem::replace(rm, new_value))
    }
}
impl<T: Clone> Rx<T> {
    pub fn get_cloned(&self) -> T {
        self.visit(Clone::clone)
    }
}
impl<T: Copy> Rx<T> {
    pub fn get(&self) -> T {
        self.get_cloned()
    }
}
pub struct RxGuard<'a, T> {
    guard: RwLockReadGuard<'a, T>,
}
impl<'a, T> Deref for RxGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}
pub struct RxGuardMutBase<'a, T, const SILENT: bool> {
    guard: RwLockWriteGuard<'a, T>,
    rx: &'a Rx<T>,
}
pub type RxGuardMut<'a, T> = RxGuardMutBase<'a, T, false>;
pub type RxGuardMutSilent<'a, T> = RxGuardMutBase<'a, T, true>;

impl<'a, T, const SILENT: bool> Deref for RxGuardMutBase<'a, T, SILENT> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}
impl<'a, T, const SILENT: bool> DerefMut for RxGuardMutBase<'a, T, SILENT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}
impl<'a, T, const SILENT: bool> Drop for RxGuardMutBase<'a, T, SILENT> {
    fn drop(&mut self) {
        if !SILENT {
            // TODO: Use less strict ordering if possible
            self.rx.version.fetch_add(1, Ordering::SeqCst);
            let locked = self.rx.subscriptions.lock().unwrap();
            locked.wake_all();
        }
    }
}

mod awaitable {
    use std::{
        pin::Pin,
        sync::atomic::Ordering,
        task::{Context, Poll},
    };

    use super::Rx;
    use crate::subscriptions::SubscriptionKey;
    use futures::{Future, Stream, StreamExt};
    pub struct RxChangeStream<'a, T> {
        key: Option<SubscriptionKey>,
        rx: &'a Rx<T>,
        version: usize,
    }

    impl<T> Rx<T> {
        pub fn listen<'a>(&'a self) -> RxChangeStream<'a, T> {
            RxChangeStream {
                key: None,
                rx: self,
                version: self.version.load(Ordering::SeqCst),
            }
        }
        pub async fn for_each_async<'a, R: Future<Output = ()>, F: FnMut(&T) -> R>(
            &'a self,
            mut func: F,
        ) {
            let stream = self.listen();
            self.visit(|t| func(t)).await;
            stream.for_each(|_| self.visit(|t| func(t))).await
        }
        pub async fn for_each<'a, F: FnMut(&T)>(&'a self, mut func: F) {
            self.for_each_async(|t| {
                func(t);
                async {}
            })
            .await
        }
    }

    impl<'a, T> Stream for RxChangeStream<'a, T> {
        type Item = ();
        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            if self.key.is_none() {
                let key = self
                    .rx
                    .subscriptions
                    .lock()
                    .unwrap()
                    .add(cx.waker().to_owned());
                self.key = Some(key);
            }
            let version = self.rx.version.load(Ordering::SeqCst);
            if version > self.version {
                self.version = version;
                Poll::Ready(Some(()))
            } else {
                Poll::Pending
            }
        }
    }
    impl<'a, T> Future for RxChangeStream<'a, T> {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            match self.poll_next(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => Poll::Ready(()),
            }
        }
    }
    impl<'a, T> Drop for RxChangeStream<'a, T> {
        fn drop(&mut self) {
            if let Some(key) = self.key.take() {
                self.rx.subscriptions.lock().unwrap().remove(key)
            }
        }
    }
}

pub use awaitable::RxChangeStream;
