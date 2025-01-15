use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use web_todomvc::app::app;

fn main() {
    #[cfg(feature = "ssr")]
    {
        use async_ui_web::render_to_string;
        let v = render_to_string(app());

        let v = futures_lite::future::block_on(v);
        println!("block_on finished");
        println!("{v}");
    }
}
