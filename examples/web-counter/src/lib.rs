use async_ui_web::{
    components::{button, text, ButtonProp},
    fragment, mount, Fragment,
};
use observables::{cell::ReactiveCell, ObservableAsExt};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    mount(root());
    Ok(())
}
async fn root() {
    counter().await;
}
async fn counter() {
    let value = ReactiveCell::new(0);

    fragment((
        button([
            ButtonProp::Children(Fragment::from((text(&"decrement"),))),
            ButtonProp::OnPress(&mut |_ev| {
                *value.borrow_mut() -= 1;
            }),
        ]),
        text(&value.as_observable().map(|v| format!("count = {v}"))),
        button([
            ButtonProp::Children(Fragment::from((text(&"increment"),))),
            ButtonProp::OnPress(&mut |_ev| {
                *value.borrow_mut() += 1;
            }),
        ]),
    ))
    .await;
}
