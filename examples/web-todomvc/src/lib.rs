use async_ui_web::mount;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

mod app;
use app::app;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    mount(app());
    Ok(())
}
