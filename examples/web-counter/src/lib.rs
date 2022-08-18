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
    .await;
}
async fn counter() {
    let value = ObservableCell::new(0);

    fragment![
        Button {
            children: fragment![Text { text: &"decrement" }],
            on_press: &mut |_ev| {
                {
                    *value.borrow_mut() -= 1;
                }
            },
            ..Default::default()
        },
        Text {
            text: &value.as_observable().map(|v| format!("{}", v))
        },
        Button {
            children: fragment![Text { text: &"increment" }],
            on_press: &mut |_ev| {
                *value.borrow_mut() += 1;
            },
            ..Default::default()
        },
        async {
            loop {
                value.as_observable().until_change().await;
                web_sys::console::log_1(&"hello".into());
            }
        }
    ]
    .await;
}
