/**
A future that does nothing, pending forever.

This is for rendering things without children, for example
```
# use async_ui_web::components::Span;
# use async_ui_web::NoChild;
# let _ = async {
    let span = Span::new();
    span.render(NoChild).await;
# };
```

Functionally, this is no different from what [std::future::pending] provides.
*/
pub struct NoChild;
impl std::future::Future for NoChild {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Pending
    }
}
