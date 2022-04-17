use std::{cell::Cell, future::Future, rc::Rc, task::Poll};

use async_ui_reactive::local::Rx;
use async_ui_utils::Join;
use async_ui_web::{create_portal, hidable, list, mount, Render};
use async_ui_web_html::{anchor, button, div, span, text};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    mount((my_component(),));
    Ok(())
}
async fn my_component() {
    let (p_ent, p_ext) = create_portal();
    Render::from((
        div().children((text().content("hello world"),)),
        counter(),
        div().children((text().content("hi"), p_ext.render(), text().content("bye"))),
        p_ent.render((
            text().content("oh my"),
            text().content("confusion"),
            anchor()
                .href("https://example.com")
                .on_click(|ev| ev.prevent_default())
                .children((text().content("qwerqwer"),)),
        )),
        list_test(),
        take_children((hidable_test(),)),
    ))
    .await;
}
async fn hidable_test() {
    let switch = Rx::new(true);
    Join::from((
        hidable(&switch, (text().content("i may be hidden"),)),
        async {
            Timeout::new(1000).await;
            *switch.borrow_mut() = false;
            Timeout::new(1000).await;
            *switch.borrow_mut() = true;
        },
    ))
    .await;
}
async fn counter() {
    let value = Rx::new(0);
    let content = Rx::new("0".into());
    Render::from((
        button()
            .on_click(|_ev| {
                value.visit_mut(|m| *m -= 1);
                content.replace(value.get().to_string());
            })
            .children((text().content("-"),)),
        span().children((text().content_reactive(&content),)),
        button()
            .on_click(|_ev| {
                value.visit_mut(|m| *m += 1);
                content.replace(value.get().to_string());
            })
            .children((text().content("+"),)),
    ))
    .await;
}
async fn list_test() {
    let children = Rx::new(vec![0, 2, 3]);
    let child_factory = |key: &i32| Render::from((text().content(&key.to_string()),));
    Join::from((list(&children, child_factory), async {
        Timeout::new(1000).await;
        children.borrow_mut().push(4);
        Timeout::new(1000).await;
        children.borrow_mut().insert(1, 1);
    }))
    .await;
}

async fn take_children(children: impl Into<Render<'_>>) {
    Render::from((div().children((
        text().content("below is my children"),
        div().children((children.into(),)),
    )),))
    .await
}

struct Timeout {
    inner: TimeoutInner,
}
impl Timeout {
    pub fn new(duration: u32) -> Self {
        Self {
            inner: TimeoutInner::Duration(duration),
        }
    }
}
enum TimeoutInner {
    Duration(u32),
    Fired(Rc<Cell<bool>>),
    Null,
}
impl Future for Timeout {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.inner = match std::mem::replace(&mut self.inner, TimeoutInner::Null) {
            TimeoutInner::Duration(d) => {
                let fired = Rc::new(Cell::new(false));
                let fired_copy = fired.clone();
                let waker = cx.waker().to_owned();
                let clos = Closure::once_into_js(move || {
                    fired_copy.set(true);
                    waker.wake()
                });
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &clos.as_ref().unchecked_ref(),
                        d as i32,
                    )
                    .expect("setTimeout failed");
                TimeoutInner::Fired(fired)
            }
            TimeoutInner::Fired(f) => {
                if f.get() {
                    return Poll::Ready(());
                }
                TimeoutInner::Fired(f)
            }
            null => null,
        };
        Poll::Pending
    }
}
