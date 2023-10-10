use std::{future::Future, marker::PhantomData};

/// Provides Future extension methods useful for writing UI.
///
/// Implemented for every `Future` type.
pub trait UiFutureExt: Future + Sized {
    /// Turn the given future into one that never finishes.
    ///
    /// You can specify the output type of the output future to be anything,
    /// because the future will never produce output.
    ///
    /// ```rust
    /// # async fn my_async_fn() {}
    /// # let _ = async {
    /// use async_ui_web_core::combinators::UiFutureExt;
    /// let fut = my_async_fn().pend_after::<std::convert::Infallible>();
    /// fut.await; // will never finish
    /// # };
    /// ```
    ///
    /// `f.pend_after()` is equivalent to
    /// ```rust
    /// # let f = async {};
    /// # async { let _: i32 = 
    /// async {
    /// 	f.await;
    /// 	std::future::pending().await
    /// }
    /// # .await; };
    /// ```
    fn pend_after<T>(self) -> PendAfter<Self, T> {
        PendAfter {
            future: Some(self),
            _phantom: PhantomData,
        }
    }
    /// Run some Future while this one is running.
    ///
    /// `f.meanwhile(g)` is equivalent to
    /// ```rust
    /// # let (f, g) = (async {}, async {});
    /// # use async_ui_web_core::combinators::{UiFutureExt, race};
    /// # async {
    /// race((
    /// 	f,
    /// 	g.pend_after()
    /// ))
    /// # };
    /// ```
    /// 
    /// Use this to display UI as side-effect of some async execution.
    /// For example, `load_data().meanwhile(spinner()).await`.
    fn meanwhile<F: Future>(
        self,
        effect: F,
    ) -> <(Self, PendAfter<F, Self::Output>) as super::race::Race>::Future {
        use super::race::Race;
        (self, effect.pend_after()).race()
    }
}

impl<F: Future> UiFutureExt for F {}

#[pin_project::pin_project]
pub struct PendAfter<F: Future, T> {
    #[pin]
    future: Option<F>,
    _phantom: PhantomData<T>,
}

impl<F: Future, T> Future for PendAfter<F, T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();
        if let Some(fut) = this.future.as_mut().as_pin_mut() {
            if fut.poll(cx).is_ready() {
                this.future.set(None);
            }
        }
        std::task::Poll::Pending
    }
}
