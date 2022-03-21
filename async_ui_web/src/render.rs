use std::{
    future::{pending, Future},
    marker::PhantomPinned,
    pin::Pin,
    rc::Rc,
    task::Poll,
};

use async_ui_spawn::{
    check_drop_guarantee,
    wasm::{start_executor, SpawnedFuture, TaskWrapper},
    RootSpawnWrappedFuture,
};
use web_sys::Node;

pub use crate::control::node_guard::NodeGuard;
use crate::{
    control::element_control::{ElementControl, ELEMENT_CONTROL},
    control::vnode::{NodeVNode, VNode},
    element::Element,
};

// pub(crate) async fn render_with_control<'e>(
//     mut children: Vec<Element<'e>>,
//     control: ElementControl,
// ) {
//     if children.len() == 1 {
//         let mut child = children.pop().unwrap();
//         child.set_control(control);
//         let mut fut = SpawnedFuture::new(child.to_boxed_future());
//         let _task = unsafe { fut.launch_and_get_task() };
//         pending().await
//     } else {
//         let mut tasks = Vec::with_capacity(children.len());
//         for (index, mut child) in children.into_iter().enumerate() {
//             child.set_control(control.nest(index));
//             let mut fut = SpawnedFuture::new(child.to_boxed_future());
//             let task = unsafe { fut.launch_and_get_task() };
//             tasks.push(task);
//         }
//         pending().await
//     }
// }

pin_project_lite::pin_project! {
    pub struct RenderFuture<'e> {
        inner: RenderFutureInner<'e>,
        _pin: PhantomPinned
    }
}

enum RenderFutureInner<'e> {
    Start {
        children: Vec<Element<'e>>,
        control: Option<ElementControl>,
    },
    LaunchedOne {
        _task: TaskWrapper<'e>,
    },
    LaunchedMany {
        _tasks: Vec<TaskWrapper<'e>>,
    },
    Null,
}
pub(crate) fn render_with_control(
    children: Vec<Element<'_>>,
    control: Option<ElementControl>,
) -> RenderFuture<'_> {
    RenderFuture {
        inner: RenderFutureInner::Start { children, control },
        _pin: PhantomPinned,
    }
}
impl<'e> Future for RenderFuture<'e> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.inner = match std::mem::replace(&mut self.inner, RenderFutureInner::Null) {
            RenderFutureInner::Start {
                mut children,
                control,
            } => {
                let control = control.unwrap_or_else(|| ELEMENT_CONTROL.with(Clone::clone));
                check_drop_guarantee(&self);
                if children.len() == 1 {
                    let mut child = children.pop().unwrap();
                    child.set_control(control);
                    let mut fut = SpawnedFuture::new(child.to_boxed_future());
                    let task = unsafe { fut.launch_and_get_task() };
                    RenderFutureInner::LaunchedOne { _task: task }
                } else {
                    let tasks = children
                        .into_iter()
                        .enumerate()
                        .map(|(index, mut child)| {
                            child.set_control(control.nest(index));
                            let mut fut = SpawnedFuture::new(child.to_boxed_future());
                            let task = unsafe { fut.launch_and_get_task() };
                            task
                        })
                        .collect();
                    RenderFutureInner::LaunchedMany { _tasks: tasks }
                }
            }
            x => x,
        };

        Poll::Pending
    }
}

pub fn render(children: Vec<Element<'_>>) -> RenderFuture<'_> {
    render_with_control(children, None)
}

pub fn put_node(node: Node) -> NodeGuard {
    ELEMENT_CONTROL.with(|ctr| ctr.put_node(node))
}

pub fn render_in_node(children: Vec<Element<'_>>, node: Node) -> RenderFuture<'_> {
    let control = ElementControl::new_with_vnode(Rc::new(VNode::from(NodeVNode::new(node))));
    render_with_control(children, Some(control))
}

pub fn mount_at(root: Element<'static>, node: Node) {
    wasm_bindgen_futures::spawn_local(RootSpawnWrappedFuture::new(async move {
        web_sys::console::log_1(&"spawn".into());
        let fut = render_in_node(vec![root], node);
        let mut fut = SpawnedFuture::new(Box::pin(fut));
        let _task = unsafe { fut.launch_and_get_task() };
        start_executor();
        pending::<()>().await;
    }));
}

pub fn mount(root: Element<'static>) {
    let node = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap();
    mount_at(root, node.into());
}
