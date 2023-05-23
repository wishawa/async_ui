use async_ui_web::{
    components::{Button, Text},
    join,
    prelude_traits::*,
    select,
};
pub async fn counter() {
    let mut value = 0;
    let value_text = Text::new();
    let incr_button = Button::new();
    let decr_button = Button::new();
    join((
        value_text.render(),
        incr_button.render("+1".render()),
        decr_button.render("-1".render()),
        async {
            loop {
                value_text.set_data(&value.to_string());
                select! {
                    _ = incr_button.until_click() => {
                        value += 1;
                    }
                    _ = decr_button.until_click() => {
                        value -= 1;
                    }
                }
            }
        },
    ))
    .await;
}
