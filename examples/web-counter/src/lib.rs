use async_ui_web::{
    components::{Button, Text},
    fragment, mount,
};
use observables::{cell::ObservableCell, ObservableExt};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    mount(fragment![root()]);
    Ok(())
}
async fn root() {
    fragment![
        (Text {
            text: &"hello world"
        }),
        counter()
    ]
    .await
}
async fn counter() {
    let value = &ObservableCell::new(0);
    let on_press_decr = |_ev| *value.borrow_mut() -= 1;
    let on_press_incr = |_ev| *value.borrow_mut() += 1;
    let display_text = value.map(|v| format!("{}", v));

    fragment![
        Button {
            children: fragment![Text { text: &"decrement" }],
            on_press: &on_press_decr,
            ..Default::default()
        },
        Text {
            text: &display_text
        },
        Button {
            children: fragment![Text { text: &"increment" }],
            on_press: &on_press_incr,
            ..Default::default()
        },
        async {
            loop {
                (&value).until_change().await;
                web_sys::console::log_1(&"hello".into());
            }
        }
    ]
    .await
}
