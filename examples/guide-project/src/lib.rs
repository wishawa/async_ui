#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    async_ui_web::mount(app());
}

async fn app() {
    // UI code goes here
}

mod building_ui;
mod dynamicity;
mod events;
