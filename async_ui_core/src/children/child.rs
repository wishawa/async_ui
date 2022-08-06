use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use scoped_async_spawn::{SpawnGuard, SpawnedTask};

use crate::{backend::BackendTrait, vnode::VNode};

use super::spawner::Spawner;

trait ChildInnerTrait<'c, B>: 'c
where
    B: BackendTrait,
{
    fn spawn(
        &mut self,
        vnode: Rc<VNode<B>>,
        guard: Pin<&mut SpawnGuard<'c, Spawner, ()>>,
        cx: &mut Context<'_>,
    );
}
pin_project! {
    struct ElementFuture<B, F>
    where
        B: BackendTrait,
        F: Future<Output = ()>
    {
        #[pin]
        future: F,
        vnode: Rc<VNode<B>>
    }
}
impl<B, F> Future for ElementFuture<B, F>
where
    B: BackendTrait,
    F: Future<Output = ()>,
{
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        B::get_vnode_key().set(&this.vnode, || this.future.poll(cx))
    }
}
impl<'c, B, F> ChildInnerTrait<'c, B> for ChildInner<'c, F>
where
    Self: 'c,
    B: BackendTrait,
    F: Future<Output = ()>,
{
    fn spawn(
        &mut self,
        vnode: Rc<VNode<B>>,
        guard: Pin<&mut SpawnGuard<'c, Spawner, ()>>,
        cx: &mut Context<'_>,
    ) {
        match std::mem::replace(self, Self::Null) {
            ChildInner::NotMounted { component } => {
                let mut spawned = guard.spawn_task(ElementFuture {
                    future: component,
                    vnode,
                });
                let _ = Pin::new(&mut spawned).poll(cx);
                *self = Self::Mounted { _spawned: spawned };
            }
            _ => unreachable!(),
        }
    }
}
enum ChildInner<'c, F>
where
    F: Future<Output = ()>,
{
    Null,
    NotMounted {
        component: F,
    },
    Mounted {
        _spawned: SpawnedTask<'c, Spawner, ()>,
    },
}
pub struct Child<'c, B>
where
    B: BackendTrait,
{
    inner: Box<dyn ChildInnerTrait<'c, B>>,
}

impl<'c, B> Child<'c, B>
where
    B: BackendTrait,
{
    pub fn new<I>(future: I) -> Self
    where
        I: IntoFuture<Output = ()>,
        I::IntoFuture: 'c,
    {
        let component = future.into_future();
        Self {
            inner: Box::new(ChildInner::NotMounted { component }),
        }
    }
    pub(super) fn mount(
        &mut self,
        vnode: Rc<VNode<B>>,
        guard: Pin<&mut SpawnGuard<'c, Spawner, ()>>,
        cx: &mut Context<'_>,
    ) {
        self.inner.spawn(vnode, guard, cx);
    }
}
