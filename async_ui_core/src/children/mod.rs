mod child;
use std::{future::Future, pin::Pin, task::Poll};

use pin_project_lite::pin_project;
use scoped_async_spawn::boxed::ScopeSafeBox;

use crate::backend::BackendTrait;
use child::PreSpawnChild;

pub mod for_macro {
    pub use super::child::PreSpawnChild;
    pub use super::Children;
    pub use scoped_async_spawn::{boxed::ScopeSafeBox, SpawnedFuture};

    #[macro_export]
    macro_rules! children {
        [$($ch:expr),*] => {
            $crate::for_macro::Children::from(::std::vec![
                $(
                    $crate::for_macro::PreSpawnChild::new($ch)
                ),*
            ])
        };
    }
}

pin_project! {
    #[project = ChildrenEnumProj]
    enum ChildrenInner<'c, B: BackendTrait> {
        Created {
            futures: Vec<PreSpawnChild<'c, B>>,
        },
        Polled {
            // TODO:	The dyn Fuuture here is always SpawnedFuture, which has fixed size.
            // 			Ideally we would avoid the double indirection.
            #[pin] futures: ScopeSafeBox<[ScopeSafeBox<dyn Future<Output = ()> + 'c>]>,
            been_polled: bool
        }
    }
}
pin_project! {
    pub struct Children<'c, B: BackendTrait> {
        #[pin] inner: ChildrenInner<'c, B>
    }
}
impl<'c, B: BackendTrait> From<Vec<PreSpawnChild<'c, B>>> for Children<'c, B> {
    fn from(futures: Vec<PreSpawnChild<'c, B>>) -> Self {
        Self {
            inner: ChildrenInner::Created { futures },
        }
    }
}
impl<'c, B: BackendTrait> Future for Children<'c, B> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut outer_this = self.project();
        let this = outer_this.inner.as_mut().project();
        match this {
            ChildrenEnumProj::Created { futures } => {
                let futures = ScopeSafeBox::from_boxed(
                    std::mem::take(futures)
                        .into_iter()
                        .map(|ele| ele.convert(B::get_vnode_key().with(Clone::clone)))
                        .collect(),
                );
                outer_this.inner.set(ChildrenInner::Polled {
                    futures,
                    been_polled: false,
                });
            }
            ChildrenEnumProj::Polled {
                futures,
                been_polled,
            } => {
                if !*been_polled {
                    *been_polled = true;
                    futures.with_scope(|p: Pin<&mut [ScopeSafeBox<dyn Future<Output = ()>>]>| {
                        // Taken from https://github.com/rust-lang/futures-rs/blob/556cc461be75316dcc00b37ec2b887f1a039a8d2/futures-util/src/future/join_all.rs#L18
                        fn iter_pin_mut<T>(
                            slice: Pin<&mut [T]>,
                        ) -> impl Iterator<Item = Pin<&mut T>> {
                            // Safety: `std` _could_ make this unsound if it were to decide Pin's
                            // invariants aren't required to transmit through slices. Otherwise this has
                            // the same safety as a normal field pin projection.
                            unsafe { slice.get_unchecked_mut() }
                                .iter_mut()
                                .map(|t| unsafe { Pin::new_unchecked(t) })
                        }
                        iter_pin_mut(p).for_each(|f| {
                            let _ = f.poll(cx);
                        });
                    });
                }
            }
        }
        Poll::Pending
    }
}
