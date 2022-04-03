use std::{borrow::Cow, cell::Cell, future::Future, rc::Rc, task::Poll};

use async_ui_reactive::Rx;
use async_ui_utils::{join, vec_into};
use async_ui_web::{create_portal, hidable, list, mount, render};
use async_ui_web_html::{anchor, button, div, span, text};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    mount(my_component().into());
    Ok(())
}
async fn my_component() {
    let (p_ent, p_ext) = create_portal();
    render(vec_into![
        div().children(vec_into![text().content("hello world")]),
        counter(),
        div().children(vec_into![
            text().content("hi"),
            p_ext.to_element(),
            text().content("bye")
        ]),
        p_ent.to_element(vec_into![
            text().content("oh my"),
            text().content("confusion"),
            anchor()
                .href("https://example.com")
                .on_click(|ev| ev.prevent_default())
                .children(vec_into![text().content("link")])
        ]),
        list_test(),
        hidable_test()
    ])
    .await;
}
async fn hidable_test() {
    let switch = Rx::new(true);
    join(
        hidable(&switch, vec_into![text().content("i may be hidden")]),
        async {
            Timeout::new(1000).await;
            *switch.borrow_mut() = false;
            Timeout::new(1000).await;
            *switch.borrow_mut() = true;
        },
    )
    .await;
}
async fn counter() {
    let value = Rx::new(0);
    let content = Rx::new(Cow::from("0"));
    join(
        render(vec_into![
            button()
                .on_click(|_ev| {
                    value.visit_mut(|m| *m -= 1);
                })
                .children(vec_into![text().content("-")]),
            span().children(vec_into![text().content_reactive(&content)]),
            button()
                .on_click(|_ev| {
                    value.visit_mut(|m| *m += 1);
                })
                .children(vec_into![text().content("+")]),
        ]),
        value.for_each(|n| {
            content.replace(n.to_string().into());
        }),
    )
    .await;
}
async fn list_test() {
    let children = Rx::new(vec![
        (1, Some(text().content("1").into())),
        (2, Some(text().content("3").into())),
        (4, Some(text().content("5").into())),
    ]);
    join(list(&children), async {
        Timeout::new(1000).await;
        children
            .borrow_mut()
            .push((5, Some(text().content("this is new!").into())));
        Timeout::new(1000).await;
        children
            .borrow_mut()
            .insert(1, (3, Some(text().content("inserted").into())));
    })
    .await;
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
