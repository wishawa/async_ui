use std::{
    any::{Any, TypeId},
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::{backend::BackendTrait, context::ContextMap, position::PositionIndex, vnode::VNode};

use super::VNodeTrait;

pub struct ContextVNode<B: BackendTrait> {
    parent: Rc<VNode<B>>,
    context: ContextMap,
}

impl<B: BackendTrait> ContextVNode<B> {
    pub fn new(parent: Rc<VNode<B>>, context: ContextMap) -> Self {
        Self { parent, context }
    }
}

impl<B: BackendTrait> VNodeTrait<B> for ContextVNode<B> {
    fn add_child_node(&self, node: B::Node, position: PositionIndex) {
        self.parent.add_child_node(node, position)
    }

    fn del_child_node(&self, position: PositionIndex) {
        self.parent.del_child_node(position)
    }

    fn get_context_map<'s>(&'s self) -> &'s ContextMap {
        &self.context
    }
}

pub fn get_context<B: BackendTrait, T: 'static>() -> Rc<T> {
    B::get_vnode_key().with(|vn| {
        let cmap = vn.get_context_map();
        let type_id = TypeId::of::<T>();
        let entry = cmap
            .inner
            .get(&type_id)
            .expect("Context not set.")
            .to_owned();
        let val = entry.downcast::<T>().unwrap();
        val
    })
}
enum WithContextState<B>
where
    B: BackendTrait,
{
    NotStarted { value: Rc<dyn Any> },
    Started { vnode: Rc<VNode<B>> },
    Null,
}

pin_project! {
    pub struct WithContext<B, F>
    where
        B: BackendTrait,
        F: Future
    {
        #[pin]
        future: F,
        state: WithContextState<B>,
    }
}
impl<B, F> Future for WithContext<B, F>
where
    B: BackendTrait,
    F: Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let vk = B::get_vnode_key();
        let vnode = match std::mem::replace(this.state, WithContextState::Null) {
            WithContextState::NotStarted { value } => {
                let parent = vk.with(Clone::clone);
                let context = ContextMap {
                    inner: parent
                        .get_context_map()
                        .inner
                        .update(value.type_id(), value),
                };
                let vnode = Rc::new(ContextVNode::new(parent, context).into());
                vnode
            }
            WithContextState::Started { vnode } => vnode,
            _ => unreachable!(),
        };
        let res = vk.set(&vnode, || this.future.poll(cx));
        *this.state = WithContextState::Started { vnode };
        res
    }
}
impl<B, F> WithContext<B, F>
where
    B: BackendTrait,
    F: Future,
{
    pub fn new<T: 'static, I: IntoFuture<IntoFuture = F>>(into_future: I, value: Rc<T>) -> Self {
        Self {
            future: into_future.into_future(),
            state: WithContextState::NotStarted { value },
        }
    }
}
