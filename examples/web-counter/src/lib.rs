use async_ui_web::{mount, children, components::Text};
use wasm_bindgen::{
    prelude::{wasm_bindgen,},
     JsValue,
};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    mount(children![
        root()
    ]);
    Ok(())
}
async fn root() {
    (Text {
        text: "hello world"
    }).await
}
