#[allow(dead_code)]
mod input_in_app {
    // ANCHOR: input-in-app
    use async_ui_web::html::Input; // 👈 import the Input component

    // 👇 this should be the same `app` function in `src/lib.rs` from the project setup
    async fn app() {
        let my_input = Input::new();
        my_input.render().await;
    }
    // ANCHOR_END: input-in-app
}
#[allow(dead_code)]
mod input_component {
    use async_ui_web::html::Input;
    // ANCHOR: input-component
    // 👇 This is your first component 💯
    async fn my_input_field() {
        let my_input = Input::new();
        my_input.render().await;
    }

    async fn app() {
        my_input_field().await; // 👈 use the component you made
    }
    // ANCHOR_END: input-component
}
