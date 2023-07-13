impl super::nodes::Input {
    pub fn new_with_type(ty: &str) -> Self {
        let input = Self::new();
        input.set_type(ty);
        input
    }
}

macro_rules! make_input_types {
    ($fn_name:ident, $text:literal) => {
        impl super::nodes::Input {
            pub fn $fn_name() -> Self {
                Self::new_with_type($text)
            }
        }
    };
}

make_input_types!(new_button, "button");
make_input_types!(new_checkbox, "checkbox");
make_input_types!(new_color, "color");
make_input_types!(new_date, "date");
make_input_types!(new_datetime, "datetime");
make_input_types!(new_email, "email");
make_input_types!(new_file, "file");
make_input_types!(new_hidden, "hidden");
make_input_types!(new_image, "image");
make_input_types!(new_month, "month");
make_input_types!(new_number, "number");
make_input_types!(new_password, "password");
make_input_types!(new_radio, "radio");
make_input_types!(new_range, "range");
make_input_types!(new_reset, "reset");
make_input_types!(new_search, "search");
make_input_types!(new_submit, "submit");
make_input_types!(new_tel, "tel");
make_input_types!(new_text, "text");
make_input_types!(new_time, "time");
make_input_types!(new_url, "url");
make_input_types!(new_week, "week");
