use async_ui_web::prelude_traits::*;

pub async fn hello_world() {
    "Hello World!".render().await;
}
