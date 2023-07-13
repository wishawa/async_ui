use std::{
    cell::RefCell,
    collections::HashMap,
    future::{poll_fn, Future},
    rc::Rc,
};

use async_ui_web::{
    event_traits::{EmitEditEvent, EmitElementEvent},
    html::{Anchor, Button, Div, Input, Label, Li, Paragraph, Span, Ul, H1},
    join,
    lists::DynamicList,
    race, select,
    shortcut_traits::{ShortcutClassList, ShortcutRenderStr},
    NoChild, ReactiveCell,
};
use futures_lite::FutureExt;
use wasm_bindgen::UnwrapThrowExt;

type Id = u32;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterMode {
    All,
    Active,
    Completed,
}

impl FilterMode {
    fn display_text(&self) -> &'static str {
        match self {
            FilterMode::All => "All",
            FilterMode::Active => "Active",
            FilterMode::Completed => "Completed",
        }
    }
    fn should_show(&self, checked: bool) -> bool {
        matches!(
            (self, checked),
            (Self::All, _) | (Self::Active, false) | (Self::Completed, true)
        )
    }
}
struct GlobalState {
    num_active: ReactiveCell<i32>,
    num_total: ReactiveCell<i32>,
    filter_mode: ReactiveCell<FilterMode>,
}

pub async fn app() {
    join((app_main(), info_footer())).await;
}

async fn app_main() {
    let mut id_counter: Id = 0;
    let global_state = &GlobalState {
        filter_mode: ReactiveCell::new(FilterMode::All),
        num_active: ReactiveCell::new(0),
        num_total: ReactiveCell::new(0),
    };

    let head_part;
    let main;
    let foot_part;

    let toggle_all = Input::new_checkbox();
    const TOGGLE_ALL_ID: &str = "toggle-all";
    toggle_all.set_id(TOGGLE_ALL_ID);
    toggle_all.add_class(style::toggle_all);
    let toggle_all_label = Label::new();
    toggle_all_label.set_html_for(TOGGLE_ALL_ID);
    let list_wrapper = Ul::new();
    list_wrapper.add_class(style::todo_list);

    let list = DynamicList::new();
    let items = RefCell::new(HashMap::<Id, Rc<TodoItem>>::new());
    struct ListAndItems<L, I> {
        list: L,
        items: I,
    }
    let list_and_items = Rc::new(ListAndItems { list, items });
    {
        let todoapp = Div::new();
        todoapp.add_class(style::todoapp);
        todoapp
    }
    .render(join((
        {
            head_part = HeadPart::new();
            &head_part
        }
        .render(),
        {
            main = Div::new();
            main.add_class(style::main);
            &main
        }
        .render(join((
            toggle_all.render(),
            toggle_all_label.render("Mark all as complete".render()),
            list_wrapper.render(list_and_items.list.render()),
            {
                foot_part = FootPart::new(global_state);
                &foot_part
            }
            .render(),
        ))),
        async {
            loop {
                toggle_all.until_change().await;
                let checked = toggle_all.checked();
                for (_id, item) in list_and_items.items.borrow().iter() {
                    item.set_checked(checked);
                }
            }
        },
        async {
            loop {
                foot_part.until_clear_completed().await;
                list_and_items.items.borrow_mut().retain(|id, item| {
                    let completed = *item.checked.borrow();
                    if completed {
                        list_and_items.list.remove(id);
                    }
                    !completed
                });
            }
        },
        async {
            loop {
                let text = head_part.until_add().await;
                let todo = Rc::new(TodoItem::new(global_state, text));
                let id: Id = id_counter;
                let weak = Rc::downgrade(&list_and_items);
                let todo_rc = todo.clone();
                let item_fut = (async move {
                    todo_rc.render_until_deleted().await;
                    if let Some(list_and_items) = weak.upgrade() {
                        list_and_items.items.borrow_mut().remove(&id);
                        list_and_items.list.remove(&id);
                    }
                })
                .boxed_local();
                list_and_items.list.insert(id, item_fut, None);
                list_and_items.items.borrow_mut().insert(id, todo);
                id_counter += 1;
            }
        },
        async {
            loop {
                global_state.num_active.until_change().await;
                let all_completed = *global_state.num_active.borrow() == 0;
                toggle_all.set_checked(all_completed);
            }
        },
        async {
            loop {
                main.set_class(style::hidden, *global_state.num_total.borrow() == 0);
                global_state.num_total.until_change().await;
            }
        },
    )))
    .await;
}

struct HeadPart {
    input: Input,
}
impl HeadPart {
    fn new() -> Self {
        let input = Input::new();
        input.add_class(style::new_todo);
        input.set_placeholder("What needs to be done?");
        input.set_autofocus(true);
        Self { input }
    }
    async fn render(&self) {
        let wrapper;
        {
            wrapper = Div::new();
            &wrapper
        }
        .render(join((
            H1::new().render("todos".render()),
            self.input.render(),
        )))
        .await;
    }
    async fn until_add(&self) -> String {
        loop {
            self.input.until_change().await;
            let text = self.input.value();
            let text = text.trim();
            if !text.is_empty() {
                self.input.set_value("");
                return text.to_string();
            }
        }
    }
}
struct FootPart<'a> {
    global: &'a GlobalState,
    clear_completed: Button,
}
impl<'a> FootPart<'a> {
    fn new(global: &'a GlobalState) -> Self {
        let this = Self {
            global,
            clear_completed: Button::new(),
        };
        this.clear_completed.add_class(style::clear_completed);
        this
    }
    async fn render(&self) {
        let num_left_span;
        {
            let wrapper = Div::new();
            wrapper.add_class(style::footer);
            wrapper
        }
        .render(join((
            {
                num_left_span = Span::new();
                num_left_span.add_class(style::todo_count);
                &num_left_span
            }
            .render(NoChild),
            async {
                loop {
                    let num_left = *self.global.num_active.borrow();
                    if num_left == 1 {
                        num_left_span.set_inner_text("1 item left");
                    } else {
                        num_left_span.set_inner_text(&format!("{num_left} items left"));
                    }
                    self.global.num_active.until_change().await;
                }
            },
            {
                let ul = Ul::new();
                ul.add_class(style::filters);
                ul
            }
            .render(join(
                [FilterMode::All, FilterMode::Active, FilterMode::Completed]
                    .map(|mode| filter_button(self.global, mode)),
            )),
            self.clear_completed.render("Clear completed".render()),
            async {
                loop {
                    let should_show =
                        *self.global.num_total.borrow() > *self.global.num_active.borrow();
                    self.clear_completed.set_class(style::hidden, !should_show);
                    race((
                        self.global.num_total.until_change(),
                        self.global.num_active.until_change(),
                    ))
                    .await;
                }
            },
        )))
        .await;
    }
    async fn until_clear_completed(&self) {
        self.clear_completed.until_click().await;
    }
}

async fn filter_button(global: &GlobalState, mode: FilterMode) {
    let li = Li::new();
    let a = Anchor::new();
    join((
        li.render(a.render(mode.display_text().render())),
        async {
            loop {
                a.until_click().await;
                *global.filter_mode.borrow_mut() = mode;
            }
        },
        async {
            loop {
                a.set_class(style::selected, *global.filter_mode.borrow() == mode);
                global.filter_mode.until_change().await;
            }
        },
    ))
    .await;
}

struct TodoItem<'a> {
    text: ReactiveCell<String>,
    checked: ReactiveCell<bool>,
    global: &'a GlobalState,
}

impl<'a> TodoItem<'a> {
    fn new(global: &'a GlobalState, text: String) -> Self {
        let this = Self {
            text: ReactiveCell::new(text),
            checked: ReactiveCell::new(false),
            global,
        };
        *global.num_active.borrow_mut() += 1;
        *global.num_total.borrow_mut() += 1;
        this
    }
    async fn render_until_deleted(self: Rc<Self>) {
        let wrapper;
        let label;
        let checkbox;
        let delete;
        race((
            map_to_nothing(
                {
                    wrapper = Li::new();
                    &wrapper
                }
                .render(join((
                    {
                        let view = Div::new();
                        view.add_class(style::view);
                        view
                    }
                    .render(join((
                        {
                            checkbox = Input::new_checkbox();
                            checkbox.add_class(style::toggle);
                            &checkbox
                        }
                        .render(),
                        {
                            label = Label::new();
                            &label
                        }
                        .render(NoChild),
                        {
                            delete = Button::new();
                            delete.add_class(style::destroy);
                            &delete
                        }
                        .render(NoChild),
                    ))),
                    async {
                        loop {
                            label.until_dblclick().await;
                            wrapper.add_class(style::editing);
                            let editor = Input::new();
                            editor.add_class(style::edit);
                            editor.set_value(&self.text.borrow());
                            race((
                                editor.render(),
                                poll_fn(|_| {
                                    editor.focus().unwrap_throw();
                                    std::task::Poll::Pending
                                }),
                                map_to_nothing(editor.until_blur()),
                                map_to_nothing(editor.until_change()),
                            ))
                            .await;
                            *self.text.borrow_mut() = editor.value().trim().to_string();
                            wrapper.del_class(style::editing);
                        }
                    },
                ))),
            ),
            async {
                loop {
                    let checked = select! {
                        _ = checkbox.until_change() => {
                            let new_checked = checkbox.checked();
                            *self.checked.borrow_mut() = new_checked;
                            new_checked
                        }
                        _ = self.checked.until_change() => {
                            let checked = self.checked.borrow();
                            checkbox.set_checked(*checked);
                            *checked
                        }
                    };
                    *self.global.num_active.borrow_mut() += 1 - (2 * (checked as i32));
                    wrapper.set_class(style::completed, checked);
                }
            },
            async {
                loop {
                    label.set_inner_text(&self.text.borrow());
                    self.text.until_change().await;
                }
            },
            async {
                loop {
                    wrapper.set_class(
                        style::hidden,
                        !self
                            .global
                            .filter_mode
                            .borrow()
                            .should_show(*self.checked.borrow()),
                    );
                    race((
                        self.global.filter_mode.until_change(),
                        self.checked.until_change(),
                    ))
                    .await;
                }
            },
            async {
                delete.until_click().await;
            },
        ))
        .await;
    }
    fn set_checked(&self, checked: bool) {
        if *self.checked.borrow() != checked {
            *self.checked.borrow_mut() = checked;
        }
    }
}
impl<'a> Drop for TodoItem<'a> {
    fn drop(&mut self) {
        *self.global.num_active.borrow_mut() -= !*self.checked.borrow() as i32;
        *self.global.num_total.borrow_mut() -= 1;
    }
}

async fn info_footer() {
    let wrapper = Div::new();
    wrapper.add_class(style::info);
    wrapper
        .render(join((
            Paragraph::new().render("Double-click to edit a todo".render()),
            Paragraph::new().render(join((
                "Written with ".render(),
                {
                    let link = Anchor::new();
                    link.set_href("https://github.com/wishawa/async_ui");
                    link
                }
                .render("Async UI".render()),
                ".".render(),
            ))),
        )))
        .await;
}

async fn map_to_nothing(f: impl Future) {
    f.await;
}

mod style {
    // CSS taken from [https://todomvc.com/examples/vanilla-es6/](here).
    async_ui_web::css!(
        r#"
html,
body {
	margin: 0;
	padding: 0;
}

button {
	margin: 0;
	padding: 0;
	border: 0;
	background: none;
	font-size: 100%;
	vertical-align: baseline;
	font-family: inherit;
	font-weight: inherit;
	color: inherit;
	-webkit-appearance: none;
	appearance: none;
	-webkit-font-smoothing: antialiased;
	-moz-osx-font-smoothing: grayscale;
}

body {
	font: 14px 'Helvetica Neue', Helvetica, Arial, sans-serif;
	line-height: 1.4em;
	background: #f5f5f5;
	color: #4d4d4d;
	min-width: 230px;
	max-width: 550px;
	margin: 0 auto;
	-webkit-font-smoothing: antialiased;
	-moz-osx-font-smoothing: grayscale;
	font-weight: 300;
}

:focus {
	outline: 0;
}

.hidden {
	display: none;
}

.todoapp {
	background: #fff;
	margin: 130px 0 40px 0;
	position: relative;
	box-shadow: 0 2px 4px 0 rgba(0, 0, 0, 0.2),
	            0 25px 50px 0 rgba(0, 0, 0, 0.1);
}

.todoapp input::-webkit-input-placeholder {
	font-style: italic;
	font-weight: 300;
	color: #e6e6e6;
}

.todoapp input::-moz-placeholder {
	font-style: italic;
	font-weight: 300;
	color: #e6e6e6;
}

.todoapp input::input-placeholder {
	font-style: italic;
	font-weight: 300;
	color: #e6e6e6;
}

.todoapp h1 {
	position: absolute;
	top: -155px;
	width: 100%;
	font-size: 100px;
	font-weight: 100;
	text-align: center;
	color: rgba(175, 47, 47, 0.15);
	-webkit-text-rendering: optimizeLegibility;
	-moz-text-rendering: optimizeLegibility;
	text-rendering: optimizeLegibility;
}

.new-todo,
.edit {
	position: relative;
	margin: 0;
	width: 100%;
	font-size: 24px;
	font-family: inherit;
	font-weight: inherit;
	line-height: 1.4em;
	border: 0;
	color: inherit;
	padding: 6px;
	border: 1px solid #999;
	box-shadow: inset 0 -1px 5px 0 rgba(0, 0, 0, 0.2);
	box-sizing: border-box;
	-webkit-font-smoothing: antialiased;
	-moz-osx-font-smoothing: grayscale;
}

.new-todo {
	padding: 16px 16px 16px 60px;
	border: none;
	background: rgba(0, 0, 0, 0.003);
	box-shadow: inset 0 -2px 1px rgba(0,0,0,0.03);
}

.main {
	position: relative;
	z-index: 2;
	border-top: 1px solid #e6e6e6;
}

.toggle-all {
	width: 1px;
	height: 1px;
	border: none; /* Mobile Safari */
	opacity: 0;
	position: absolute;
	right: 100%;
	bottom: 100%;
}

.toggle-all + label {
	width: 60px;
	height: 34px;
	font-size: 0;
	position: absolute;
	top: -52px;
	left: -13px;
	-webkit-transform: rotate(90deg);
	transform: rotate(90deg);
}

.toggle-all + label:before {
	content: '❯';
	font-size: 22px;
	color: #e6e6e6;
	padding: 10px 27px 10px 27px;
}

.toggle-all:checked + label:before {
	color: #737373;
}

.todo-list {
	margin: 0;
	padding: 0;
	list-style: none;
}

.todo-list li {
	position: relative;
	font-size: 24px;
	border-bottom: 1px solid #ededed;
}

.todo-list li:last-child {
	border-bottom: none;
}

.todo-list li.editing {
	border-bottom: none;
	padding: 0;
}

.todo-list li.editing .edit {
	display: block;
	width: 506px;
	padding: 12px 16px;
	margin: 0 0 0 43px;
}

.todo-list li.editing .view {
	display: none;
}

.todo-list li .toggle {
	text-align: center;
	width: 40px;
	/* auto, since non-WebKit browsers doesn't support input styling */
	height: auto;
	position: absolute;
	top: 0;
	bottom: 0;
	margin: auto 0;
	border: none; /* Mobile Safari */
	-webkit-appearance: none;
	appearance: none;
}

.todo-list li .toggle {
	opacity: 0;
}

.todo-list li .toggle + label {
	/*
		Firefox requires '#' to be escaped - https://bugzilla.mozilla.org/show_bug.cgi?id=922433
		IE and Edge requires *everything* to be escaped to render, so we do that instead of just the '#' - https://developer.microsoft.com/en-us/microsoft-edge/platform/issues/7157459/
	*/
	background-image: url('data:image/svg+xml;utf8,%3Csvg%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%22-10%20-18%20100%20135%22%3E%3Ccircle%20cx%3D%2250%22%20cy%3D%2250%22%20r%3D%2250%22%20fill%3D%22none%22%20stroke%3D%22%23ededed%22%20stroke-width%3D%223%22/%3E%3C/svg%3E');
	background-repeat: no-repeat;
	background-position: center left;
}

.todo-list li .toggle:checked + label {
	background-image: url('data:image/svg+xml;utf8,%3Csvg%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%22-10%20-18%20100%20135%22%3E%3Ccircle%20cx%3D%2250%22%20cy%3D%2250%22%20r%3D%2250%22%20fill%3D%22none%22%20stroke%3D%22%23bddad5%22%20stroke-width%3D%223%22/%3E%3Cpath%20fill%3D%22%235dc2af%22%20d%3D%22M72%2025L42%2071%2027%2056l-4%204%2020%2020%2034-52z%22/%3E%3C/svg%3E');
}

.todo-list li label {
	word-break: break-all;
	padding: 15px 15px 15px 60px;
	display: block;
	line-height: 1.2;
	transition: color 0.4s;
}

.todo-list li.completed label {
	color: #d9d9d9;
	text-decoration: line-through;
}

.todo-list li .destroy {
	display: none;
	position: absolute;
	top: 0;
	right: 10px;
	bottom: 0;
	width: 40px;
	height: 40px;
	margin: auto 0;
	font-size: 30px;
	color: #cc9a9a;
	margin-bottom: 11px;
	transition: color 0.2s ease-out;
}

.todo-list li .destroy:hover {
	color: #af5b5e;
}

.todo-list li .destroy:after {
	content: '×';
}

.todo-list li:hover .destroy {
	display: block;
}

.todo-list li .edit {
	display: none;
}

.todo-list li.editing:last-child {
	margin-bottom: -1px;
}

.footer {
	color: #777;
	padding: 10px 15px;
	height: 20px;
	text-align: center;
	border-top: 1px solid #e6e6e6;
}

.footer:before {
	content: '';
	position: absolute;
	right: 0;
	bottom: 0;
	left: 0;
	height: 50px;
	overflow: hidden;
	box-shadow: 0 1px 1px rgba(0, 0, 0, 0.2),
	            0 8px 0 -3px #f6f6f6,
	            0 9px 1px -3px rgba(0, 0, 0, 0.2),
	            0 16px 0 -6px #f6f6f6,
	            0 17px 2px -6px rgba(0, 0, 0, 0.2);
}

.todo-count {
	float: left;
	text-align: left;
}

.todo-count strong {
	font-weight: 300;
}

.filters {
	margin: 0;
	padding: 0;
	list-style: none;
	position: absolute;
	right: 0;
	left: 0;
}

.filters li {
	display: inline;
}

.filters li a {
	color: inherit;
	margin: 3px;
	padding: 3px 7px;
	text-decoration: none;
	border: 1px solid transparent;
	border-radius: 3px;
}

.filters li a:hover {
	border-color: rgba(175, 47, 47, 0.1);
}

.filters li a.selected {
	border-color: rgba(175, 47, 47, 0.2);
}

.clear-completed,
html .clear-completed:active {
	float: right;
	position: relative;
	line-height: 20px;
	text-decoration: none;
	cursor: pointer;
}

.clear-completed:hover {
	text-decoration: underline;
}

.info {
	margin: 65px auto 0;
	color: #bfbfbf;
	font-size: 10px;
	text-shadow: 0 1px 0 rgba(255, 255, 255, 0.5);
	text-align: center;
}

.info p {
	line-height: 1;
}

.info a {
	color: inherit;
	text-decoration: none;
	font-weight: 400;
}

.info a:hover {
	text-decoration: underline;
}

/*
	Hack to remove background from Mobile Safari.
	Can't use it globally since it destroys checkboxes in Firefox
*/
@media screen and (-webkit-min-device-pixel-ratio:0) {
	.toggle-all,
	.todo-list li .toggle {
		background: none;
	}

	.todo-list li .toggle {
		height: 40px;
	}
}

@media (max-width: 430px) {
	.footer {
		height: 50px;
	}

	.filters {
		bottom: 10px;
	}
}
	"#
    );
}
