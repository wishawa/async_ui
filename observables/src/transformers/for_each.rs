use pin_project_lite::pin_project;

use crate::{ObservableAsExt, Version};

use super::super::ObservableAs;
use std::{future::Future, marker::PhantomData, task::Poll};

pin_project! {
    pub struct ForEach<W, I, H>
    where
        W: ObservableAs<I>,
        H: FnMut(&I),
        I: ?Sized,
    {
        wrapped: W,
        handler: H,
        last_version: Version,
        _phantom: PhantomData<I>
    }
}

impl<W, I, H> ForEach<W, I, H>
where
    W: ObservableAs<I>,
    H: FnMut(&I),
    I: ?Sized,
{
    pub(crate) fn new(wrapped: W, handler: H) -> Self {
        Self {
            wrapped,
            handler,
            last_version: Version::new_null(),
            _phantom: PhantomData,
        }
    }
}
impl<W, I, H> Future for ForEach<W, I, H>
where
    W: ObservableAs<I>,
    H: FnMut(&I),
    I: ?Sized,
{
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let version = this.wrapped.get_version();
        if version > *this.last_version {
            *this.last_version = version;
            this.wrapped.visit(this.handler);
            this.wrapped.add_waker(cx.waker().to_owned());
        }
        Poll::Pending
    }
}

// pin_project! {
// 	pub struct ForEachAsync<W, I, H, F>
// 	where
// 		W: ObservableAs<I>,
// 		H: FnMut(&I) -> F,
// 		F: Future<Output = ()>,
// 		I: ?Sized,
// 	{
// 		wrapped: W,
// 		handler: H,
// 		last_version: Version,
// 		lastest_future: F,
// 		_phantom: PhantomData<I>
// 	}
// }
