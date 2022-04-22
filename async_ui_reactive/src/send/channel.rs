use std::{collections::VecDeque, future::Future, rc::Rc};

use super::Rx;

type Inner<T> = Rc<Rx<VecDeque<T>>>;

pub struct Sender<T> {
    inner: Inner<T>,
}
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct Receiver<T> {
    inner: Inner<T>,
}

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Rc::new(Rx::new(VecDeque::new()));
    (
        Sender {
            inner: inner.clone(),
        },
        Receiver { inner },
    )
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) {
        self.inner.visit_mut(|buf| {
            buf.push_back(value);
        });
    }
}
impl<T> Receiver<T> {
    pub async fn receive(&self) -> T {
        if let Some(item) = self.inner.visit_mut_silent(|buf| buf.pop_front()) {
            item
        } else {
            let stream = self.inner.listen();
            stream.await;
            self.inner.visit_mut_silent(|buf| {
                buf.pop_front()
                    .expect("buffer should not be empty after change")
            })
        }
    }
    pub async fn for_each<F: FnMut(T)>(&self, mut func: F) {
        loop {
            self.inner.visit_mut(|buf| {
                buf.drain(..).for_each(|item| {
                    func(item);
                });
            });
            self.inner.listen().await;
        }
    }
    pub async fn for_each_async<R: Future<Output = ()>, F: FnMut(T) -> R>(&self, mut func: F) {
        loop {
            while let Some(item) = self.inner.visit_mut(|buf| buf.pop_front()) {
                func(item).await;
            }
            self.inner.listen().await;
        }
    }
}
