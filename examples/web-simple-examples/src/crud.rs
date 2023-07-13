use std::collections::HashMap;

use async_ui_web::{
    html::{Button, Div, Input, Label, Select},
    join,
    lists::DynamicList,
    prelude_traits::*,
    select, NoChild,
};

pub async fn crud() {
    let person_select_children = DynamicList::new();
    let person_select;
    let mut person_elements = HashMap::new();
    let filter_input;
    let name_input;
    let surname_input;
    let create_button;
    let update_button;
    let delete_button;
    let mut id_counter = 0;
    join((
        {
            let wrapper = Div::new();
            wrapper.add_class(style::vertical);
            wrapper
        }
        .render(join((
            {
                let top = Div::new();
                top.add_class(style::horizontal);
                top
            }
            .render(join((
                {
                    let left = Div::new();
                    left.add_class(style::vertical);
                    left
                }
                .render(join((
                    {
                        filter_input = LabeledField::new("filter", "Filter Prefix:");
                        &filter_input
                    }
                    .render(),
                    {
                        person_select = Select::new();
                        person_select.set_multiple(true);
                        &person_select
                    }
                    .render(person_select_children.render()),
                ))),
                {
                    let right = Div::new();
                    right.add_class(style::vertical);
                    right
                }
                .render(join((
                    {
                        name_input = LabeledField::new("person-name", "Name:");
                        &name_input
                    }
                    .render(),
                    {
                        surname_input = LabeledField::new("person-surname", "Surname:");
                        &surname_input
                    }
                    .render(),
                ))),
            ))),
            {
                let bottom = Div::new();
                bottom.add_class(style::horizontal);
                bottom
            }
            .render(join((
                {
                    create_button = Button::new();
                    &create_button
                }
                .render("Create".render()),
                {
                    update_button = Button::new();
                    update_button.set_disabled(true);
                    &update_button
                }
                .render("Update".render()),
                {
                    delete_button = Button::new();
                    delete_button.set_disabled(true);
                    &delete_button
                }
                .render("Delete".render()),
            ))),
        ))),
        async {
            loop {
                select! {
                    _ = create_button.until_click() => {
                        let name = format!(
                            "{}, {}",
                            surname_input.field.value(),
                            name_input.field.value(),
                        );
                        id_counter += 1;
                        let opt = async_ui_web::html::Option::new();
                        opt.set_value(&id_counter.to_string());
                        opt.set_inner_text(&name);
                        person_select_children.insert(id_counter, opt.render(NoChild), None);
                        person_elements.insert(id_counter, (opt, name));
                    },
                    _ = update_button.until_click() => {
                        let name = format!(
                            "{}, {}",
                            surname_input.field.value(),
                            name_input.field.value(),
                        );
                        let seek = person_select.value().parse::<i32>().unwrap();
                        let elem = person_elements.get_mut(&seek).unwrap();
                        elem.0.set_inner_text(&name);
                        elem.1 = name;
                    },
                    _ = delete_button.until_click() => {
                        let seek = person_select.value().parse::<i32>().unwrap();
                        person_elements.remove(&seek);
                        person_select_children.remove(&seek);
                    },
                    _ = person_select.until_input() => {
                        let value = person_select.value();
                        person_select.set_value(&value);
                        update_button.set_disabled(value.is_empty());
                        delete_button.set_disabled(value.is_empty());
                    },
                    _ = filter_input.field.until_input() => {
                        let prefix = filter_input.field.value();
                        for (_id, (opt, name)) in person_elements.iter() {
                            let should_hide = !name.starts_with(&prefix);
                            opt.set_disabled(should_hide);
                            opt.set_hidden(should_hide);
                        }
                    },
                }
            }
        },
    ))
    .await;
}

struct LabeledField {
    pub label: Label,
    pub field: Input,
}
impl LabeledField {
    pub fn new(id: &str, text: &str) -> Self {
        let field = Input::new_text();
        field.set_id(id);
        let label = Label::new();
        label.set_html_for(id);
        label.set_text_content(Some(text));
        Self { label, field }
    }
    pub async fn render(&self) {
        {
            let d = Div::new();
            d.add_class(style::horizontal);
            d.add_class(style::space_between);
            d
        }
        .render(join((self.label.render(NoChild), self.field.render())))
        .await;
    }
}

mod style {
    async_ui_web::css!(
        r#"
.wrapper {
	display: flex;
	flex-direction: column;
}
.vertical {
	display: flex;
	flex-direction: column;
}
.horizontal {
	display: flex;
	flex-direction: row;
}
.space-between {
	justify-content: space-between;
}
		"#
    );
}
