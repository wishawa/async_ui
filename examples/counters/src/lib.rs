use std::cell::RefCell;

use async_ui_reactive::singlethread::{create_channel, ReactiveRefCell};
use async_ui_utils::{join4, vec_into};
use async_ui_web::{create_portal, mount, render, Element};
use async_ui_web_html::{anchor, button, div, span, text};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

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
        my_other_component(vec_into![p_ext.render()]),
        div().children(vec_into![text().content("hello world")]),
        p_ent.render(vec_into![
            text().content("oh my god"),
            text().content("confusion"),
        ])
    ])
    .await;
}
async fn my_other_component(children: Vec<Element<'_>>) {
    let (decrement_ent, decrement_ext) = create_channel();
    let (increment_ent, increment_ext) = create_channel();
    let value = RefCell::new(0);
    let content = ReactiveRefCell::new("0".into());
    let (link_ent, link_ext) = create_channel();
    join4(
        render(vec_into![
            button()
                .on_click(decrement_ent)
                .children(vec_into![text().content("-")]),
            span().children(vec_into![text().content_reactive(&content)]),
            render(children),
            button()
                .on_click(increment_ent)
                .children(vec_into![text().content("+")]),
            anchor()
                .href("https://example.com")
                .on_click(link_ent)
                .children(vec_into![text().content("link")])
        ]),
        async {
            loop {
                let _ev = decrement_ext.receive().await;
                let mut v = value.borrow_mut();
                *v -= 1;
                *content.borrow_mut() = v.to_string().into();
            }
        },
        async {
            loop {
                let _ev = increment_ext.receive().await;
                let mut v = value.borrow_mut();
                *v += 1;
                *content.borrow_mut() = v.to_string().into();
            }
        },
        async {
            loop {
                let ev = link_ext.receive().await;
                ev.prevent_default();
            }
        },
    )
    .await;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
