use std::{collections::HashMap, rc::Rc};

use async_ui_web::futures_lite::FutureExt;
use async_ui_web::{
    components::{
        button, list, text, text_input, view, ButtonProp, ListModel, ListProp, TextInputProp,
        ViewProp,
    },
    fragment, mount,
    utils::class_list::ClassList,
};
use observables::{cell::ReactiveCell, Observable, ObservableAsExt};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use x_bow::{create_store, Store, Track};

#[derive(Track)]
struct State {
    current_id: TodoId,
    #[x_bow(no_track)]
    todos_list: ListModel<TodoId>,
    #[x_bow(no_track)]
    todos_map: HashMap<TodoId, Rc<Store<Todo>>>,
    #[x_bow(no_track)]
    filter: DisplayFilter,
    #[x_bow(no_track)]
    counts: Counts,
}
struct Counts {
    total: usize,
    completed: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum DisplayFilter {
    All,
    Active,
    Complete,
}

mod reducers {
    use std::{collections::hash_map::Entry, rc::Rc};

    use crate::{State, Todo, TodoId};
    use x_bow::{create_store, Store};

    pub(super) fn get_id_incremented(store: &Store<State>) -> TodoId {
        let mut bm = store.current_id.borrow_mut();
        bm.0 += 1;
        *bm
    }

    pub(super) fn add_todo(store: &Store<State>, text: String) {
        let id = get_id_incremented(store);
        store.todos_map.borrow_mut().insert(
            id,
            Rc::new(create_store(Todo {
                value: text,
                done: false,
            })),
        );
        store.todos_list.borrow_mut().insert(0, id);
        store.counts.borrow_mut().total += 1;
    }

    pub(super) fn edit_todo_value(store: &Store<State>, id: TodoId, value: String) {
        *store
            .todos_map
            .borrow()
            .get(&id)
            .unwrap()
            .value
            .borrow_mut() = value;
    }
    pub(super) fn edit_todo_done(store: &Store<State>, id: TodoId, done: bool) {
        let need_update = {
            let b = store.todos_map.borrow();
            let mut prev = b.get(&id).unwrap().done.borrow_mut();
            if *prev != done {
                *prev = done;
                true
            } else {
                false
            }
        };
        if need_update {
            let mut bm = store.counts.borrow_mut();
            if done {
                bm.completed += 1;
            } else {
                bm.completed -= 1;
            }
        }
    }

    pub(super) fn remove_todo(store: &Store<State>, id: TodoId) {
        if let Some(todo) = {
            let todo = store.todos_map.borrow_mut().remove(&id);
            todo
        } {
            let mut counts = store.counts.borrow_mut();
            if *todo.done.borrow() {
                counts.completed -= 1;
            }
            counts.total -= 1;
        }
        let mut list_model = store.todos_list.borrow_mut();
        if let Some(to_remove) = {
            list_model
                .underlying_vector()
                .iter()
                .position(|item_id| *item_id == id)
        } {
            list_model.remove(to_remove);
        }
    }
    pub(super) fn set_all_done(store: &Store<State>, done: bool) {
        let keys_vec = { store.todos_list.borrow().underlying_vector().clone() };
        {
            let mut bm = store.todos_map.borrow_mut();
            for key in keys_vec.iter() {
                *bm.get_mut(key).unwrap().done.borrow_mut() = done;
            }
        }
        store.counts.borrow_mut().completed = if done { keys_vec.len() } else { 0 };
    }
    pub(super) fn get_all_done(store: &Store<State>) -> bool {
        let counts = store.counts.borrow();
        counts.completed == counts.total
    }
    pub(super) fn clear_completed(store: &Store<State>) {
        let keys_vec = { store.todos_list.borrow().underlying_vector().clone() };
        let mut to_remove_indexes = Vec::new();
        {
            let mut bm = store.todos_map.borrow_mut();
            for (idx, key) in keys_vec.iter().enumerate() {
                match bm.entry(*key) {
                    Entry::Occupied(entry) => {
                        if *entry.get().done.borrow() {
                            entry.remove();
                            to_remove_indexes.push(idx);
                        }
                    }
                    Entry::Vacant(_) => panic!("no corresponding todo"),
                }
            }
        }
        {
            let mut counts = store.counts.borrow_mut();
            counts.total -= to_remove_indexes.len();
            counts.completed = 0;
        }
        {
            let mut bm = store.todos_list.borrow_mut();
            for idx in to_remove_indexes.into_iter().rev() {
                bm.remove(idx);
            }
        }
    }
}

#[derive(Track, Clone, Copy, PartialEq, Eq, Hash)]
struct TodoId(usize);

#[derive(Track)]
struct Todo {
    value: String,
    done: bool,
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    mount(root());
    Ok(())
}

async fn root() {
    let store = create_store(State {
        todos_map: HashMap::new(),
        todos_list: ListModel::new(),
        current_id: TodoId(0),
        filter: DisplayFilter::All,
        counts: Counts {
            completed: 0,
            total: 0,
        },
    });
    view([
        ViewProp::Children(fragment((
            header(),
            view([
                ViewProp::Children(fragment((
                    top_part(&store),
                    list_content(&store),
                    bottom_part(&store),
                ))),
                ViewProp::Class(&"main-container".into()),
            ]),
        ))),
        ViewProp::Class(&"wrapper".into()),
    ])
    .await;
}
async fn header() {
    view([
        ViewProp::Children(fragment((text(&"todos"),))),
        ViewProp::Class(&"header-box".into()),
    ])
    .await;
}
async fn bottom_part(store: &Store<State>) {
    let classes = ClassList::new(["bottom-part"]);
    view([
        ViewProp::Children(fragment((
            view([
                ViewProp::Children(fragment((active_label(store), clear_button(store)))),
                ViewProp::Class(&"bottom-labels".into()),
            ]),
            filter_bar(store),
        ))),
        ViewProp::Class(&classes),
    ])
    .or(async {
        loop {
            let hide = store.counts.borrow().total == 0;
            classes.set("hidden", hide);
            store.counts.as_observable().until_change().await;
        }
    })
    .await;
}
async fn active_label(store: &Store<State>) {
    let value = ReactiveCell::new("".into());
    view([
        ViewProp::Children(fragment((text(&value.as_observable()).or(async {
            loop {
                let count = {
                    let counts = store.counts.borrow();
                    counts.total - counts.completed
                };
                *value.borrow_mut() = if count == 1 {
                    format!("{} item left", count)
                } else {
                    format!("{} items left", count)
                };
                store.counts.as_observable().until_change().await;
            }
        }),))),
        ViewProp::Class(&"active-label-box".into()),
    ])
    .await;
}
async fn clear_button(store: &Store<State>) {
    let classes = ClassList::new(["clear-button"]);
    button([
        ButtonProp::Children(fragment((text(&"Clear Completed"),))),
        ButtonProp::OnPress(&mut |_ev| {
            reducers::clear_completed(store);
        }),
        ButtonProp::Class(&classes),
    ])
    .or(async {
        loop {
            classes.set("hidden", store.counts.borrow().completed == 0);
            store.counts.as_observable().until_change().await;
        }
    })
    .await;
}
async fn filter_bar(store: &Store<State>) {
    async fn filter_button(store: &Store<State>, filter: DisplayFilter) {
        let label = match filter {
            DisplayFilter::All => "All",
            DisplayFilter::Active => "Active",
            DisplayFilter::Complete => "Complete",
        };
        let classes = ClassList::new(["filter-button"]);
        button([
            ButtonProp::Children(fragment((text(&label),))),
            ButtonProp::Class(&classes),
            ButtonProp::OnPress(&mut |_ev| {
                *store.filter.borrow_mut() = filter;
            }),
        ])
        .or(async {
            loop {
                let set = *store.filter.borrow() == filter;
                classes.set("filter-button-selected", set);
                store.filter.as_observable().until_change().await;
            }
        })
        .await;
    }

    let buttons = fragment((
        filter_button(store, DisplayFilter::All),
        filter_button(store, DisplayFilter::Active),
        filter_button(store, DisplayFilter::Complete),
    ));
    view([
        ViewProp::Children(buttons),
        ViewProp::Class(&"filter-bar".into()),
    ])
    .await;
}
async fn list_item(store: &Store<State>, id: TodoId) {
    let handle = {
        if let Some(todo) = store.todos_map.borrow().get(&id) {
            todo.to_owned()
        } else {
            return;
        }
    };
    let done_classes = ClassList::new(["done-button"]);
    let view_classes = ClassList::new(["list-item"]);
    view([
        ViewProp::Children(fragment((
            button([
                ButtonProp::OnPress(&mut |_| {
                    let done = { !*handle.done.borrow() };
                    reducers::edit_todo_done(store, id, done);
                }),
                ButtonProp::Class(&done_classes),
            ]),
            text_input([
                TextInputProp::Text(&handle.value.as_observable()),
                TextInputProp::OnBlur(&mut |ev| {
                    reducers::edit_todo_value(store, id, ev.get_text());
                }),
                TextInputProp::Class(&"item-input".into()),
            ]),
            button([
                ButtonProp::OnPress(&mut |_ev| reducers::remove_todo(store, id)),
                ButtonProp::Class(&"delete-button".into()),
            ]),
            async {
                let done_obs = handle.done.as_observable();
                let filter_obs = store.filter.as_observable();
                loop {
                    let v = *done_obs.borrow_observable();
                    let f = *filter_obs.borrow_observable();
                    done_classes.set("done-button-done", v);
                    let visible = match (f, v) {
                        (DisplayFilter::All, _) => true,
                        (DisplayFilter::Active, false) => true,
                        (DisplayFilter::Complete, true) => true,
                        _ => false,
                    };
                    view_classes.set("hidden", !visible);
                    done_obs.until_change().or(filter_obs.until_change()).await;
                }
            },
        ))),
        ViewProp::Class(&view_classes),
    ])
    .await;
}
async fn list_content(store: &Store<State>) {
    let render = &|id| list_item(store, id);
    list([
        ListProp::Data(&store.todos_list.as_observable()),
        ListProp::Render(render),
        ListProp::Class(&"list-content".into()),
    ])
    .await;
}
async fn top_part(store: &Store<State>) {
    view([
        ViewProp::Children(fragment((toggle_all_button(store), add_input_box(store)))),
        ViewProp::Class(&"top-part".into()),
    ])
    .await;
}
async fn toggle_all_button(store: &Store<State>) {
    let classes = ClassList::new(["toggle-all-button"]);
    button([
        ButtonProp::Class(&classes),
        ButtonProp::OnPress(&mut |_ev| {
            reducers::set_all_done(store, !reducers::get_all_done(store));
        }),
    ])
    .or(async {
        loop {
            {
                let new_len = store.todos_list.borrow().len();
                classes.set("hidden", new_len == 0);
            }
            store.todos_list.as_observable().until_change().await;
        }
    })
    .or(async {
        loop {
            let all_done = reducers::get_all_done(store);
            classes.set("toggle-all-button-all-done", all_done);
            store.counts.as_observable().until_change().await;
        }
    })
    .await;
}
async fn add_input_box(store: &Store<State>) {
    let value = ReactiveCell::new(String::new());
    fragment((text_input([
        TextInputProp::Text(&value.as_observable()),
        TextInputProp::OnSubmit(&mut |ev| {
            let text = ev.get_text();
            if text.len() > 0 {
                value.borrow_mut().clear();
                reducers::add_todo(store, text);
            }
        }),
        TextInputProp::OnBlur(&mut |ev| {
            *value.borrow_mut() = ev.get_text();
        }),
        TextInputProp::Class(&"add-input".into()),
        TextInputProp::Placeholder(&"What needs to be done?"),
    ]),))
    .await;
}
