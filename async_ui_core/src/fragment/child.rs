use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use async_executor::Task;
use pin_project_lite::pin_project;
use scoped_async_spawn::SpawnGuard;

use crate::{backend::BackendTrait, executor::spawn_local, vnode::VNode};

trait ChildInnerTrait<'c, B>: 'c
where
    B: BackendTrait,
{
    fn spawn(&mut self, vnode: Rc<VNode<B>>, guard: Pin<&mut SpawnGuard<'c>>);
}
pin_project! {
    struct ElementFuture<B, F>
    where
        B: BackendTrait,
        F: Future
    {
        #[pin]
        future: F,
        vnode: Rc<VNode<B>>
    }
}
impl<B, F> Future for ElementFuture<B, F>
where
    B: BackendTrait,
    F: Future,
{
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        B::get_vnode_key()
            .set(&this.vnode, || this.future.poll(cx))
            .map(|_| ())
    }
}
impl<'c, B, F> ChildInnerTrait<'c, B> for ChildInner<F>
where
    Self: 'c,
    B: BackendTrait,
    F: Future,
{
    fn spawn(&mut self, vnode: Rc<VNode<B>>, guard: Pin<&mut SpawnGuard<'c>>) {
        match std::mem::replace(self, Self::Null) {
            ChildInner::NotMounted { component } => {
                let fut = guard.convert_future(ElementFuture {
                    future: component,
                    vnode,
                });
                let task = spawn_local(fut);
                *self = Self::Mounted { _task: task };
            }
            _ => unreachable!(),
        }
    }
}
enum ChildInner<F>
where
    F: Future,
{
    Null,
    NotMounted { component: F },
    Mounted { _task: Task<()> },
}
pub struct Child<'c, B>
where
    B: BackendTrait,
{
    inner: Box<dyn ChildInnerTrait<'c, B>>,
}

impl<'c, B, I> From<I> for Child<'c, B>
where
    B: BackendTrait,
    I: IntoFuture,
    I::IntoFuture: 'c,
{
    fn from(future: I) -> Self {
        let component = future.into_future();
        Self {
            inner: Box::new(ChildInner::NotMounted { component }),
        }
    }
}

impl<'c, B> Child<'c, B>
where
    B: BackendTrait,
{
    pub(super) fn mount(&mut self, vnode: Rc<VNode<B>>, guard: Pin<&mut SpawnGuard<'c>>) {
        self.inner.spawn(vnode, guard);
    }
}
