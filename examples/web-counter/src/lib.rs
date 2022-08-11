use async_ui_web::{
    children,
    components::{Button, Text},
    mount,
};
use observables::{cell::ObservableCell, ObservableExt};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    mount(children![root()]);
    Ok(())
}
async fn root() {
    children![
        (Text {
            text: "hello world"
        }),
        counter()
    ]
    .await
}
async fn counter() {
    let value = ObservableCell::new(0);
    children![
        Button {
            children: children![Text { text: "decrement" }],
            on_press: |_ev| { *value.borrow_mut() -= 1 }
        },
        Text {
            text: (&value).map(|v| format!("{}", v))
        },
        Button {
            children: children![Text { text: "increment" }],
            on_press: |_ev| { *value.borrow_mut() += 1 }
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
