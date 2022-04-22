use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use async_ui_gtk::{mount_and_present, Render};
use async_ui_gtk_widgets::{list_view, ListViewItems, Wrappable};
use async_ui_reactive::local::Rx;
use async_ui_utils::Join;
use gtk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::{ButtonExt, EntryExt, WidgetExt},
    Application, ApplicationWindow,
};
fn main() {
    let app = Application::builder()
        .application_id("org.gtk-rs.example")
        .build();

    let build_ui = |app: &Application| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("TodoMVC")
            .build();
        mount_and_present((root(),), window);
    };
    app.connect_activate(build_ui);
    app.run();
}
struct State {
    todos: ListViewItems<Todo>,
}
impl State {
    fn new() -> Self {
        Self {
            todos: ListViewItems::new(),
        }
    }
    fn add_todo(&self, content: String) {
        self.todos.append(Todo {
            content,
            completed: false,
        });
    }
}
struct Todo {
    content: String,
    completed: bool,
}
async fn root() {
    let state = State::new();
    Render::from((gtk::Box::new(gtk::Orientation::Vertical, 0)
        .wrap()
        .children((input(&state), todos_list(&state))),))
    .await
}
async fn input(state: &State) {
    use gtk::prelude::EditableExt;
    Render::from((gtk::Entry::new()
        .wrap()
        .visit_mut(|e| e.set_placeholder_text(Some("what needs to be done?")))
        .on_activate(|entry| {
            if entry.text_length() > 0 {
                let text = entry.text().to_string();
                state.add_todo(text);
                entry.set_text("");
            }
        }),))
    .await
}
async fn todos_list(state: &State) {
    let factory = |todo: Rc<Rx<Todo>>| {
        println!("factory run");
        Render::from((todo_item(todo, state),))
    };
    Render::from((gtk::ScrolledWindow::new()
        .wrap()
        .visit_mut(|v| {
            v.set_height_request(400);
            v.set_width_request(400);
        })
        .children((list_view(&state.todos, factory),)),))
    .await;
}
async fn todo_item(todo: Rc<Rx<Todo>>, state: &State) {
    let text = Rx::new(String::new());
    Join::from((
        Render::from((gtk::Box::new(gtk::Orientation::Horizontal, 4)
            .wrap()
            .children((
                gtk::Label::new(None).wrap().text_reactive(&text),
                gtk::Button::new()
                    .wrap()
                    .visit_mut(|btn| btn.set_icon_name("edit-delete"))
                    .on_clicked(|_btn| {
                        todo.visit_mut(|td| td.content = String::from("lolz"));
                    }),
            )),)),
        todo.for_each(|td| {
            text.replace(td.content.clone());
        }),
    ))
    .await;
}
