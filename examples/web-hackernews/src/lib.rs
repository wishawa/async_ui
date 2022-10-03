use async_ui_web::mount;
use hackernews::root;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    mount(root());
    Ok(())
}
