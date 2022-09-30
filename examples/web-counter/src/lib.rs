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
        button(ButtonProp {
            children: Some(fragment((text(&"decrement"),))),
            on_press: Some(&mut |_ev| *value.borrow_mut() -= 1),
            ..Default::default()
        }),
        text(&value.as_observable().map(|v| format!("the count is {v}"))),
        button(ButtonProp {
            children: Some(fragment((text(&"increment"),))),
            on_press: Some(&mut |_ev| *value.borrow_mut() += 1),
            ..Default::default()
        }),
    ))
    .await;
}
