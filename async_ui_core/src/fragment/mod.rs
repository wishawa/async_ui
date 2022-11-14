mod child;
use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use child::Child;
use pin_project_lite::pin_project;
use scoped_async_spawn::SpawnGuard;

use crate::{backend::BackendTrait, vnode::node_pass::PassVNode};

#[doc(hidden)]
pub mod __private_macro_only {
    pub use super::child::Child;
    pub use super::Fragment;

    #[macro_export]
    macro_rules! fragment {
        [$($ch:expr),*] => {
            $crate::__private_macro_only::Fragment::new_from_vec_child(::std::vec![
                $($crate::__private_macro_only::Child::from($ch)),*
            ])
        }
    }
}

pin_project! {
    pub struct Fragment<'c, B>
    where
        B: BackendTrait
    {
        children: Vec<Child<'c, B>>,
        mounted: bool,
        #[pin]
        guard: SpawnGuard<'c>,
    }
}
impl<'c, B> Default for Fragment<'c, B>
where
    B: BackendTrait,
{
    fn default() -> Self {
        Self::new_from_vec_child(Vec::new())
    }
}
impl<'c, B> Fragment<'c, B>
where
    B: BackendTrait,
{
    pub fn new_from_vec_child(children: Vec<Child<'c, B>>) -> Self {
        Self {
            children,
            mounted: false,
            guard: SpawnGuard::new(),
        }
    }
    pub fn new_from_iter<F: IntoFuture + 'c, I: IntoIterator<Item = F>>(children: I) -> Self {
        Self::new_from_vec_child(children.into_iter().map(Child::from).collect())
    }
}

macro_rules! impl_tuple_of_children {
    ($($arg:ident=$num:tt),*) => {
        impl<'c, B: BackendTrait, $($arg : IntoFuture<Output = ()> + 'c,)*> From<($($arg,)*)> for Fragment<'c, B> {
            #[allow(unused_variables)]
            fn from(source: ($($arg,)*)) -> Self {
                crate::fragment![
                    $(source.$num),*
                ]
            }
        }
    };
}
impl_tuple_of_children!();
impl_tuple_of_children!(A0 = 0);
impl_tuple_of_children!(A0 = 0, A1 = 1);
impl_tuple_of_children!(A0 = 0, A1 = 1, A2 = 2);
impl_tuple_of_children!(A0 = 0, A1 = 1, A2 = 2, A3 = 3);
impl_tuple_of_children!(A0 = 0, A1 = 1, A2 = 2, A3 = 3, A4 = 4);
impl_tuple_of_children!(A0 = 0, A1 = 1, A2 = 2, A3 = 3, A4 = 4, A5 = 5);
impl_tuple_of_children!(A0 = 0, A1 = 1, A2 = 2, A3 = 3, A4 = 4, A5 = 5, A6 = 6);
impl_tuple_of_children!(
    A0 = 0,
    A1 = 1,
    A2 = 2,
    A3 = 3,
    A4 = 4,
    A5 = 5,
    A6 = 6,
    A7 = 7
);
impl_tuple_of_children!(
    A0 = 0,
    A1 = 1,
    A2 = 2,
    A3 = 3,
    A4 = 4,
    A5 = 5,
    A6 = 6,
    A7 = 7,
    A8 = 8
);
impl_tuple_of_children!(
    A0 = 0,
    A1 = 1,
    A2 = 2,
    A3 = 3,
    A4 = 4,
    A5 = 5,
    A6 = 6,
    A7 = 7,
    A8 = 8,
    A9 = 9
);
impl_tuple_of_children!(
    A0 = 0,
    A1 = 1,
    A2 = 2,
    A3 = 3,
    A4 = 4,
    A5 = 5,
    A6 = 6,
    A7 = 7,
    A8 = 8,
    A9 = 9,
    A10 = 10
);
impl_tuple_of_children!(
    A0 = 0,
    A1 = 1,
    A2 = 2,
    A3 = 3,
    A4 = 4,
    A5 = 5,
    A6 = 6,
    A7 = 7,
    A8 = 8,
    A9 = 9,
    A10 = 10,
    A11 = 11
);

impl<'c, B> Future for Fragment<'c, B>
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
