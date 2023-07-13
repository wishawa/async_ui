use async_ui_web::{
    html::{Button, Input, Meter, Text},
    join,
    prelude_traits::*,
    race, NoChild, ReactiveCell,
};
use futures_lite::StreamExt;

pub async fn timer() {
    // Our reactive states
    let elapsed = ReactiveCell::new(0.0);
    let total = ReactiveCell::new(15.0);

    // UI elements

    let meter = Meter::new();
    meter.set_min(0.0);
    meter.set_max(1.0);
    meter.set_value(0.0);

    let elapsed_text = Text::new();

    let slider = Input::new_range();
    slider.set_min("0.01");
    slider.set_max("30.0");
    slider.set_value("15.0");

    let reset = Button::new();

    // Join everything
    join((
        "Elapsed time: ".render(),
        meter.render(NoChild),
        elapsed_text.render(),
        "Duration: ".render(),
        slider.render(),
        reset.render("Reset".render()),
        async {
            // Keep the progress bar in sync with the progress
            loop {
                meter.set_value(*elapsed.borrow() / *total.borrow());
                race((total.until_change(), elapsed.until_change())).await;
            }
        },
        async {
            // Keep the total time in sync with the slider
            loop {
                *total.borrow_mut() = slider.value_as_number();
                slider.until_input().await;
            }
        },
        async {
            // Keep incrementing the elapsed time
            let mut stream = gloo_timers::future::IntervalStream::new(50);
            loop {
                stream.next().await;
                if *elapsed.borrow() < *total.borrow() {
                    *elapsed.borrow_mut() += 0.05;
                }
            }
        },
        async {
            // Reset when reset button is pressed
            loop {
                reset.until_click().await;
                *elapsed.borrow_mut() = 0.0;
            }
        },
        async {
            // Keep the elapsed text in sync
            loop {
                elapsed_text.set_data(&format!("{:.2}", elapsed.borrow()));
                elapsed.until_change().await;
            }
        },
    ))
    .await;
}
