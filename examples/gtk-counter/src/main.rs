use async_ui_gtk::{
    components::{Button, Text},
    fragment, mount,
};
use observables::{cell::ObservableCell, ObservableAsExt};

fn main() {
    mount(counter())
}
async fn counter() {
    let count = ObservableCell::new(0);
    fragment![
        Button {
            on_press: &mut |_| {
                *count.borrow_mut() -= 1;
            },
            children: fragment![Text { text: &"-" },],
            ..Default::default()
        },
        Text {
            text: &count.as_observable().map(|count| count.to_string())
        },
        Button {
            on_press: &mut |_| {
                *count.borrow_mut() += 1;
            },
            children: fragment![Text { text: &"+" },],
            ..Default::default()
        },
    ]
    .await;
}
