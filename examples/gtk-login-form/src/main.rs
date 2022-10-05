use std::time::Duration;

use async_ui_gtk::futures_lite::future::race;
use async_ui_gtk::{
    components::{button, text, text_input, ButtonProps, TextInputProps},
    fragment, mount,
};
use futures_timer::Delay;
use observables::{cell::ReactiveCell, ObservableAsExt};
fn main() {
    mount(root());
}
async fn root() {
    login_flow().await;
    text(&"You're in!").await;
}
async fn login_flow() {
    loop {
        let (username, password) = login_form().await;
        if check_login(username, password).await {
            // Login successful!
            break;
        } else {
            race(
                // Render the popup component.
                invalid_login_popup(),
                // Race with a future that will complete in 5 seconds.
                // In 5 seconds, this future will "win" the race and cause
                // the popup future to be dropped, unmounting the popup.
                Delay::new(Duration::from_secs(5)),
            )
            .await;
            // Loop back to the login form!
        }
    }
}
async fn check_login(username: String, password: String) -> bool {
    username == "wisha" && password == "hunter2"
}
async fn invalid_login_popup() {
    text(&"login invalid :(").await;
}
async fn login_form() -> (String, String) {
    let mut username = String::new();
    let mut password = String::new();
    let done = ReactiveCell::new(false);
    race(
        fragment((
            text_input(TextInputProps {
                on_blur: Some(&mut |ev| username = ev.get_text()),
                placeholder: Some(&"Username"),
                ..Default::default()
            }),
            text_input(TextInputProps {
                on_blur: Some(&mut |ev| password = ev.get_text()),
                placeholder: Some(&"Password"),
                ..Default::default()
            }),
            button(ButtonProps {
                children: fragment((text(&"submit"),)),
                on_press: Some(&mut |_ev| {
                    *done.borrow_mut() = true;
                }),
                ..Default::default()
            }),
        )),
        done.as_observable().until_change(),
    )
    .await;
    (username, password)
}
