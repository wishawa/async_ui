use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
    task::Poll,
};

use pin_project_lite::pin_project;
use scoped_async_spawn::{boxed::ScopeSafeBox, SpawnedFuture};

use crate::{backend::BackendTrait, executor::spawn_local, vnode::VNode};

trait ElementFutureTrait<'c, B: BackendTrait>: Future<Output = ()> {
    fn to_dyn_spawned_future(self: Box<Self>) -> Box<dyn Future<Output = ()> + 'c>;
    fn set_vnode(&mut self, vnode: Rc<VNode<B>>);
}
pin_project! {
    struct ElementFuture<B: BackendTrait, F: Future<Output = ()>> {
        #[pin] fut: F,
        vnode: Option<Rc<VNode<B>>>
    }
}

impl<B: BackendTrait, F: Future<Output = ()>> Future for ElementFuture<B, F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let vn = this
            .vnode
            .as_ref()
            .expect("VNode should have been set before mount");
        B::get_vnode_key().set(vn, || this.fut.poll(cx))
    }
}

impl<'c, B: BackendTrait, F: Future<Output = ()> + 'c> ElementFutureTrait<'c, B>
    for ElementFuture<B, F>
{
    fn to_dyn_spawned_future(self: Box<Self>) -> Box<dyn Future<Output = ()> + 'c> {
        Box::new(SpawnedFuture::new(*self, spawn_local)) as _
    }

    fn set_vnode(&mut self, vnode: Rc<VNode<B>>) {
        self.vnode = Some(vnode);
    }
}

pub struct PreSpawnChild<'c, B: BackendTrait>(Box<dyn ElementFutureTrait<'c, B> + 'c>);

impl<'c, B: BackendTrait> PreSpawnChild<'c, B> {
    pub fn new<F: IntoFuture<Output = ()> + 'c>(fut: F) -> Self {
        Self(Box::new(ElementFuture {
            fut: fut.into_future(),
            vnode: None,
        }))
    }
    pub(super) fn convert(
        mut self,
        vnode: Rc<VNode<B>>,
    ) -> ScopeSafeBox<dyn Future<Output = ()> + 'c> {
        self.0.set_vnode(vnode);
        ScopeSafeBox::from_boxed(self.0.to_dyn_spawned_future())
    }
}
