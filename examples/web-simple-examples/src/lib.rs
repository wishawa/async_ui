use async_ui_web::{
    components::{Anchor, Div},
    join, mount,
    prelude_traits::*,
};
use std::future::Future;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod cells;
mod circles;
mod clock;
mod counter;
mod crud;
mod flight;
mod hello_world;
mod temp_converter;
mod timer;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    mount(everything());
    Ok(())
}

async fn container(f: impl Future, is_7guis: bool) {
    let d = Div::new();
    d.add_class(style::wrapper);
    d.render(join((
        async {
            if is_7guis {
                let div = Div::new();
                div.add_class(style::label_7_guis);
                div.render(join((
                    "Part of the ".render(),
                    {
                        let x = Anchor::new();
                        x.set_href("https://eugenkiss.github.io/7guis/tasks");
                        x.render("7 GUIs".render())
                    },
                    " tests".render(),
                )))
                .await;
            }
        },
        f,
    )))
    .await;
}

pub async fn everything() {
    join((
        "Async UI Demo".render(),
        container(hello_world::hello_world(), false),
        container(counter::counter(), true),
        container(clock::digital(), false),
        container(clock::analog(), false),
        container(temp_converter::converter(), true),
        container(flight::flight(), true),
        container(timer::timer(), true),
        container(crud::crud(), true),
        container(circles::circles(), true),
        container(cells::cells(), true),
    ))
    .await;
}

mod style {
    async_ui_web::css!(
        r#"
.wrapper {
    padding: 2em;
    border: 1px solid black;
    position: relative;
    margin: 1em;
}
.label-7-guis {
    position: absolute;
    top: 0.5em;
    right: 0.5em;
}
"#
    );
}
