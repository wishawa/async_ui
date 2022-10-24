use std::future::pending;

use async_ui_web::{
    animation::Animator,
    components::{text, view, ElementFuture, ViewProps},
    fragment,
    futures_lite::FutureExt,
    mount, web_sys, Fragment, DOCUMENT,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::HtmlElement;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    mount(app());
    Ok(())
}

async fn app() {
    fragment((
    view(ViewProps {
        children: Fragment::new_from_iter(
            (0..64).map(|i| one_dot((i as f64) * std::f64::consts::PI / 16.0)),
        ),
        class: Some(&"container".into()),
        ..Default::default()
    }),
    text(&"
        This example showcases Async UI's \"escape hatch\" that allows you to work directly with web_sys when needed.
        The animation is driven by Rust through web_sys, scheduled with requestAnimationFrame.
        Each dot is a div with its own async loop.
    ")
))
    .await;
}

async fn one_dot(phase: f64) {
    let node = DOCUMENT.with(|doc| doc.create_element("div").expect("create element failed"));
    ElementFuture::new(pending::<()>(), node.clone().into())
        .or(async {
            let animator = Animator::new();
            let node: HtmlElement = node.unchecked_into();
            node.class_list().add_1("dot").expect("add class failed");
            loop {
                let ts = animator.next_frame().await;
                node.style()
                    .set_property(
                        "transform",
                        &format!(
                            "translateY({}vh)",
                            7.5 * (std::f64::consts::PI * ts / 1000.0 + phase).sin()
                        ),
                    )
                    .expect("set style failed");
            }
        })
        .await;
}
