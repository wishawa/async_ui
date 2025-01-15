#[cfg(feature = "csr")]
use async_ui_web::mount;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

pub mod app;
use app::app;

#[wasm_bindgen(start)]
#[cfg(feature = "csr")]
pub fn run() -> Result<(), JsValue> {
    mount(app());
    Ok(())
}
