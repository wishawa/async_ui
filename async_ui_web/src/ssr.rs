use std::future::Future;
use std::pin::{pin, Pin};
use std::task::{ready, Poll};

use async_ui_web_core::dom::{create_ssr_element, SsrNode};
use futures_lite::future::poll_fn;

use crate::executor;

pub async fn render_to_string<F: Future + 'static>(child_future: F) -> String {
    let node = AsRef::<SsrNode>::as_ref(&create_ssr_element("#root")).clone();
    let mut root_fut_own =
        async_ui_web_core::ContainerNodeFuture::new_root(child_future, node.clone());
    let mut root_fut = unsafe { Pin::new_unchecked(&mut root_fut_own) };
    executor::poll_until_loaded(async {
        println!("Before");
        poll_fn(|cx| {
            ready!(root_fut.as_mut().poll(cx));
            Poll::Ready(Some(()))
        })
        .await;
        println!("After");
    })
    .await;
    println!("serialize");
    // inner to strip outer <#root>
    let out = node.to_inner_html();
    println!("done");
    drop(root_fut_own);
    println!("dropped fut");
    out
}
