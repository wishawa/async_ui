pub mod app;

#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() -> Result<(), wasm_bindgen::JsValue> {
    use async_ui_web::mount;

    mount(app::app());
    Ok(())
}
