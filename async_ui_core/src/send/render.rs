use std::{future::Future, marker::PhantomPinned, pin::Pin, task::Poll};

use smallvec::SmallVec;

pub use super::control::node_guard::NodeGuard;
use super::{
    backend::{Backend, Spawner},
    control::{vnode::VNode, Control},
    drop_check::check_drop_scope,
    element::Element,
};

pin_project_lite::pin_project! {
    pub struct RenderFuture<'e, B>
    where B: Backend
    {
        children: Vec<Element<'e, B>>,
        control: Option<Control<B>>,
        tasks: SmallVec<[<B::Spawner as Spawner>::Task; 1]>,
        _pin: PhantomPinned
    }
}

pub fn render_with_control<'e, B: Backend>(
    children: Vec<Element<'e, B>>,
    control: Option<Control<B>>,
) -> RenderFuture<'e, B> {
    RenderFuture {
        tasks: SmallVec::new(),
        children,
        control,
        _pin: PhantomPinned,
    }
}
impl<'e, B: Backend> Future for RenderFuture<'e, B> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let ptr = &*self as *const _ as *const ();
        let this = self.project();
        if !this.children.is_empty() {
            check_drop_scope(ptr);
            let control = this
                .control
                .take()
                .unwrap_or_else(|| B::get_tls().with(Clone::clone));

            this.tasks
                .extend(this.children.drain(..).enumerate().map(|(idx, mut child)| {
                    child.set_control(control.nest(idx));
                    let task = unsafe { child.spawn() };
                    task
                }));
            *this.children = Vec::new();
        }
        Poll::Pending
    }
}
pub unsafe fn spawn_with_control<'e, B: Backend>(
    mut child: Element<'e, B>,
    control: Option<Control<B>>,
) -> <B::Spawner as Spawner>::Task {
    if let Some(ctr) = control {
        child.set_control(ctr);
    }
    unsafe { child.spawn() }
}

pub fn put_node<B: Backend>(node: <B::VNode as VNode>::Node) -> NodeGuard<B> {
    B::get_tls().with(|vn| vn.put_node(node))
}
