use std::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

use crate::shared::{ReactiveInner, SubscriptionKey};

use super::{IMCell, LockReadGuard, LockWriteGuard};

pub struct Reactive<T>(IMCell<ReactiveInner<T>>);

pub struct ReactiveGuard<'a, T> {
    guard: LockReadGuard<'a, ReactiveInner<T>>,
}
pub struct ReactiveGuardMut<'a, T> {
    guard: LockWriteGuard<'a, ReactiveInner<T>>,
}
impl<'a, T> Drop for ReactiveGuardMut<'a, T> {
    fn drop(&mut self) {
        self.guard.notify_modified();
    }
}
impl<'a, T> Deref for ReactiveGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let inner = self.guard.deref();
        inner.deref_value()
    }
}
impl<'a, T> Deref for ReactiveGuardMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let inner = self.guard.deref();
        inner.deref_value()
    }
}
impl<'a, T> DerefMut for ReactiveGuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let inner = self.guard.deref_mut();
        inner.deref_value_mut()
    }
}
impl<T> Reactive<T> {
    pub fn new(value: T) -> Self {
        Self(IMCell::new(ReactiveInner::new(value)))
    }
    pub fn borrow(&self) -> ReactiveGuard<'_, T> {
        ReactiveGuard {
            guard: self.0.lock_read(),
        }
    }
    pub fn borrow_mut(&self) -> ReactiveGuardMut<'_, T> {
        ReactiveGuardMut {
            guard: self.0.lock_write(),
        }
    }
    pub fn borrow_next(&self) -> ReactiveNextFuture<'_, T> {
        ReactiveNextFuture {
            reactive: self,
            subscribed: None,
            version: 0,
        }
    }
    pub fn borrow_next_mut(&self) -> ReactiveNextMutFuture<'_, T> {
        ReactiveNextMutFuture {
            reactive: self,
            subscribed: None,
            version: 0,
        }
    }
}
pub struct ReactiveNextFuture<'a, T> {
    reactive: &'a Reactive<T>,
    subscribed: Option<SubscriptionKey>,
    version: usize,
}
impl<'a, T> Future for ReactiveNextFuture<'a, T> {
    type Output = ReactiveGuard<'a, T>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut locked = self.reactive.0.lock_write();
        self.subscribed = if let Some(key) = self.subscribed.take() {
            let lv = locked.get_version();
            if lv > self.version {
                self.version = lv;
                locked.remove_subscription(key);
                drop(locked);
                let guard = self.reactive.0.lock_read();
                return Poll::Ready(ReactiveGuard { guard });
            }
            Some(key)
        } else {
            Some(locked.add_subscription(cx.waker().to_owned()))
        };
        Poll::Pending
    }
}
impl<'a, T> Drop for ReactiveNextFuture<'a, T> {
    fn drop(&mut self) {
        if let Some(sub) = self.subscribed.take() {
            self.reactive.0.lock_write().remove_subscription(sub);
        }
    }
}

pub struct ReactiveNextMutFuture<'a, T> {
    reactive: &'a Reactive<T>,
    subscribed: Option<SubscriptionKey>,
    version: usize,
}
impl<'a, T> Future for ReactiveNextMutFuture<'a, T> {
    type Output = ReactiveGuardMut<'a, T>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut locked = self.reactive.0.lock_write();
        self.subscribed = if let Some(key) = self.subscribed.take() {
            let lv = locked.get_version();
            if lv > self.version {
                self.version = lv;
                locked.remove_subscription(key);
                return Poll::Ready(ReactiveGuardMut { guard: locked });
            }
            Some(key)
        } else {
            Some(locked.add_subscription(cx.waker().to_owned()))
        };
        Poll::Pending
    }
}
impl<'a, T> Drop for ReactiveNextMutFuture<'a, T> {
    fn drop(&mut self) {
        if let Some(sub) = self.subscribed.take() {
            self.reactive.0.lock_write().remove_subscription(sub);
        }
    }
}
