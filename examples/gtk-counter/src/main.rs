use std::time::Duration;

use async_ui_gtk::{
    components::{Button, Text, TextInput},
    fragment, mount, Fragment,
};
use observables::{cell::ObservableCell, ObservableAsExt};
use rand::Rng;

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
        input_test(),
        sequence_test(),
    ]
    .await;
}
async fn input_test() {
    let text = ObservableCell::new(String::new());
    fragment![
        TextInput {
            on_blur: &mut |ev| {
                *text.borrow_mut() = ev.get_text();
            },
            ..Default::default()
        },
        Text {
            text: &text.as_observable()
        }
    ]
    .await;
}

fn sequence_test() -> Fragment<'static> {
    fragment![
        delayed("0"),
        delayed("1"),
        delayed("2"),
        delayed("3"),
        delayed("4"),
        delayed("5"),
        delayed("6"),
        delayed("7"),
    ]
}
async fn delayed(label: &str) {
    let mut rng = rand::thread_rng();
    let seconds = rng.gen_range(1..5);
    futures_timer::Delay::new(Duration::from_secs(seconds)).await;
    fragment![Text { text: &label }, Text { text: &"---" },].await;
}
