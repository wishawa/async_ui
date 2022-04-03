use std::time::Duration;

use async_ui_gtk::{mount, render};
use async_ui_gtk_widgets::label;
use async_ui_utils::vec_into;

fn main() {
    mount(my_component().into(), "async_ui.test_app");
    println!("mounted");
}
async fn my_component() {
    async_ui_utils::race(render(vec_into![label("hello world")]), async {
        smol::Timer::after(Duration::from_secs(3)).await;
    })
    .await;
    render(vec_into![
        label("hah"),
        label("this is getting out of hand")
    ])
    .await;
}
