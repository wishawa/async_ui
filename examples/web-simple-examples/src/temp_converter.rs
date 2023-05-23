use async_ui_web::{components::Input, join, prelude_traits::*};

pub async fn converter() {
    let input_c = Input::new();
    let input_f = Input::new();
    join((
        input_c.render(),
        "Celsius = ".render(),
        input_f.render(),
        "Farenheight".render(),
        async {
            loop {
                input_c.until_input().await;
                let Ok(v)= input_c.value().parse::<f32>() else {continue};
                input_f.set_value(&((v / 5.0 * 9.0 + 32.0).round() as i32).to_string());
            }
        },
        async {
            loop {
                input_f.until_input().await;
                let Ok(v)= input_f.value().parse::<f32>() else {continue};
                input_c.set_value(&(((v - 32.0) / 9.0 * 5.0).round() as i32).to_string());
            }
        },
    ))
    .await;
}
