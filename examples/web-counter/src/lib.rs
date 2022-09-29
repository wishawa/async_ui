use async_ui_web::{
    components::{Button, ButtonProp, Text},
    fragment, mount, Fragment,
};
use observables::{cell::ReactiveCell, ObservableAsExt};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    mount(fragment![root()]);
    Ok(())
}
async fn root() {
    counter().await;
}
async fn counter() {
    let value = ReactiveCell::new(0);

    Fragment::from((
        Button([
            ButtonProp::Children(Fragment::from((Text(&"decrement"),))),
            ButtonProp::OnPress(&mut |_ev| {
                *value.borrow_mut() -= 1;
            }),
        ]),
        Text(&value.as_observable().map(|v| format!("count = {v}"))),
        Button([
            ButtonProp::Children(Fragment::from((Text(&"increment"),))),
            ButtonProp::OnPress(&mut |_ev| {
                *value.borrow_mut() += 1;
            }),
        ]),
    ))
    .await;
}
