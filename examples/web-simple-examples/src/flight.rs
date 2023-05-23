use async_ui_web::{
    components::{self, Button, Input, Select},
    join,
    prelude_traits::*,
    race,
};
use time::Date;

pub async fn flight() {
    let date_format = time::macros::format_description!("[year]-[month]-[day]");

    let type_chooser = Select::new();
    type_chooser.set_value("one-way");
    let input_1 = Input::new();
    let input_2 = Input::new();
    input_1.add_class(style::book_date);
    input_2.add_class(style::book_date);
    let book = Button::new();

    join((
        // The UI
        type_chooser.render(join((
            {
                let x = components::Option::new();
                x.set_value("one-way");
                x.render("One-Way Flight".render())
            },
            {
                let x = components::Option::new();
                x.set_value("two-way");
                x.render("Return Flight".render())
            },
        ))),
        input_1.render(),
        input_2.render(),
        book.render("Book!".render()),
        // Code
        async {
            loop {
                // Parse inputs
                let two_way = type_chooser.value() == "two-way";
                let date_1 = Date::parse(&input_1.value(), &date_format).ok();
                let date_2 = Date::parse(&input_2.value(), &date_format).ok();

                // Disable or enable second input field
                input_2.set_disabled(!two_way);

                // Disable or enable book button
                book.set_disabled(match (two_way, date_1, date_2) {
                    (true, Some(d1), Some(d2)) if d2 > d1 => false,
                    (false, Some(_d1), _) => false,
                    _ => true,
                });

                // Set class to color the text field
                for (input, date) in [(&input_1, &date_1), (&input_2, &date_2)] {
                    input.set_class(style::invalid, date.is_none());
                }

                // Wait to do everything again on change
                race((
                    type_chooser.until_input(),
                    input_1.until_input(),
                    input_2.until_input(),
                ))
                .await;
            }
        },
    ))
    .await;
}

mod style {
    async_ui_web::css!(
        r#"
.book-date.invalid:not([disabled]) {
    background-color: orange;
}
        "#
    );
}
