mod child;
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use child::Child;
use pin_project_lite::pin_project;
use scoped_async_spawn::SpawnGuard;

use crate::{backend::BackendTrait, vnode::PassVNode};

#[doc(hidden)]
pub mod __private_macro_only {
    pub use super::child::Child;
    pub use super::Children;

    #[macro_export]
    macro_rules! children {
        [$($ch:expr),*] => {
            $crate::__private_macro_only::Children::new(::std::vec![
                $($crate::__private_macro_only::Child::new($ch)),*
            ])
        }
    }
}

pin_project! {
    pub struct Children<'c, B>
    where
        B: BackendTrait
    {
        children: Vec<Child<'c, B>>,
        mounted: bool,
        #[pin]
        guard: SpawnGuard<'c>
    }
}

impl<'c, B> Children<'c, B>
where
    B: BackendTrait,
{
    pub fn new(children: Vec<Child<'c, B>>) -> Self {
        Self {
            children,
            mounted: false,
            guard: SpawnGuard::new(),
        }
    }
}
impl<'c, B> Future for Children<'c, B>
where
    B: BackendTrait,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let parent_vnode = B::get_vnode_key().with(Clone::clone);
        if !*this.mounted {
            *this.mounted = true;
            if this.children.len() > 1 {
                this.children.iter_mut().enumerate().for_each(|(idx, ch)| {
                    let vnode = Rc::new(PassVNode::new(parent_vnode.clone(), idx).into());
                    ch.mount(vnode, this.guard.as_mut());
                });
            } else if let Some(ch) = this.children.first_mut() {
                ch.mount(parent_vnode, this.guard);
            }
        }
        Poll::Pending
    }
}
