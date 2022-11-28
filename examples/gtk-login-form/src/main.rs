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
    // Make the user log in
    login_flow().await;

    // They're logged in!
    text(&["You're in!"]).await;
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
    // This is not a popup. I haven't implemented popups yet.
    text(&["login invalid :("]).await;
}
async fn login_form() -> (String, String) {
    let mut username = String::new();
    let mut password = String::new();
    let done = ReactiveCell::new(false);
    race(
        fragment((
            // Username input
            text_input(TextInputProps {
                on_blur: &mut |ev| username = ev.get_text(),
                placeholder: &["Username"],
                ..Default::default()
            }),
            // Password input
            text_input(TextInputProps {
                on_blur: &mut |ev| password = ev.get_text(),
                placeholder: &["Password"],
                ..Default::default()
            }),
            // Submit button
            button(ButtonProps {
                children: fragment((text(&["submit"]),)),
                on_press: &mut |_ev| {
                    // When user press submit, set done to true...
                    *done.borrow_mut() = true;
                },
                ..Default::default()
            }),
        )),
        // When done is changed, this future will completes,
        // causing the race to conclude.
        // The function will then return.
        done.as_observable().until_change(),
    )
    .await;
    (username, password)
}
