use std::cell::RefCell;
use std::future::Future;
use std::pin::{pin, Pin};
use std::task::{Context, Poll};

pub fn get_executor() -> &'static () {
    todo!("tokio/other executor abstraction is wanted here.")
}

/// If something needs to be run to completion before sending user html - it needs to be wrapped in `run_loading` call.
///
/// let data = run_loading(load_data()).await;
///
/// DataDisplay::new(data).render().await
///
/// Future will be pooled until all run_loading futures are resolved
pub async fn run_loading<V>(f: impl Future<Output = V>) -> V {
    CTX.with_borrow_mut(|ctx| {
        let ctx = ctx
            .as_mut()
            .expect("ctx should be set by UntilLoadedFuture (before)");
        ctx.loading += 1;
    });
    let res = f.await;
    CTX.with_borrow_mut(|ctx| {
        let ctx = ctx
            .as_mut()
            .expect("ctx should be set by UntilLoadedFuture (after)");
        ctx.loading -= 1;
    });
    res
}

struct SsrContext {
    loading: usize,
}

thread_local! {
    static CTX: RefCell<Option<SsrContext>> = const { RefCell::new(None) };
}

#[pin_project::pin_project]
struct UntilLoadedFuture<F> {
    #[pin]
    inner: F,
    ctx: Option<SsrContext>,
    ran_once: bool,
}
impl<F: Future<Output = ()>> Future for UntilLoadedFuture<F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("POLL!");
        let project = self.project();
        let ctx = project
            .ctx
            .take()
            .expect("ctx is returned in between polls");
        CTX.with_borrow_mut(move |v| {
            assert!(v.is_none(), "CTX is empty between polls");
            *v = Some(ctx);
        });

        let poll = project.inner.poll(cx);

        let ctx = CTX.with_borrow_mut(move |v| v.take().expect("nothing should retake our ctx"));

        if ctx.loading == 0 {
            println!("Force ready!");
            // We don't care about everything not needed for first contentful load.
            return Poll::Ready(());
        }

        // TODO: This is not panic-safe, but I'm not sure how panics can be handled here yet.
        *project.ctx = Some(ctx);
        *project.ran_once = true;

        match poll {
            Poll::Ready(_) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub(crate) async fn poll_until_loaded(inner: impl Future<Output = ()>) {
    let fut = UntilLoadedFuture {
        inner,
        ctx: Some(SsrContext { loading: 0 }),
        ran_once: false,
    };
    fut.await
}
