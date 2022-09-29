use async_ui_web::{
    components::{button, text, ButtonProp},
    fragment, mount,
};
use observables::{cell::ReactiveCell, ObservableAsExt};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    mount(counter());
    Ok(())
}
async fn counter() {
    let value = ReactiveCell::new(0);
    fragment((
        button([
            ButtonProp::Children(fragment((text(&"decrement"),))),
            ButtonProp::OnPress(&mut |_ev| *value.borrow_mut() -= 1),
        ]),
        text(&value.as_observable().map(|v| format!("the count is {v}"))),
        button([
            ButtonProp::Children(fragment((text(&"increment"),))),
            ButtonProp::OnPress(&mut |_ev| *value.borrow_mut() += 1),
        ]),
    ))
    .await;
}
