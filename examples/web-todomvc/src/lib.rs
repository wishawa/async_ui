use std::{collections::HashMap, rc::Rc};

use async_ui_web::components::{radio_button, radio_group, RadioGroupProps, RadioProps};
use async_ui_web::futures_lite::FutureExt;
use async_ui_web::{
    components::{
        button, list, text, text_input, view, ButtonProps, ListModel, ListProps, TextInputProps,
        ViewProps,
    },
    fragment, mount,
    utils::class_list::ClassList,
};
use observables::{cell::ReactiveCell, ObservableAsExt};
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

#[derive(Track, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
    view(ViewProps {
        children: fragment((
            header(),
            view(ViewProps {
                children: fragment((top_part(&store), list_content(&store), bottom_part(&store))),
                class: Some(&"main-container".into()),
                ..Default::default()
            }),
            footer(),
        )),
        class: Some(&"wrapper".into()),
        ..Default::default()
    })
    .await;
}
async fn header() {
    view(ViewProps {
        children: fragment((text(&["todos"]),)),
        class: Some(&"header-box".into()),
        ..Default::default()
    })
    .await;
}

async fn top_part(store: &Store<State>) {
    async fn toggle_all_button(store: &Store<State>) {
        let classes = ClassList::new(["toggle-all-button"]);
        button(ButtonProps {
            class: Some(&classes),
            on_press: Some(&mut |_ev| {
                reducers::set_all_done(store, !reducers::get_all_done(store));
            }),
            ..Default::default()
        })
        .or(async {
            loop {
                {
                    let new_len = store.todos_list.borrow().len();
                    classes.set("toggle-all-button-disabled", new_len == 0);
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
        fragment((text_input(TextInputProps {
            text: Some(&value.as_observable()),
            on_submit: Some(&mut |ev| {
                let text = ev.get_text();
                if text.len() > 0 {
                    value.borrow_mut().clear();
                    reducers::add_todo(store, text);
                }
            }),
            on_blur: Some(&mut |ev| {
                *value.borrow_mut() = ev.get_text();
            }),
            class: Some(&"add-input".into()),
            placeholder: Some(&["What needs to be done?"]),
            ..Default::default()
        }),))
        .await;
    }
    view(ViewProps {
        children: fragment((toggle_all_button(store), add_input_box(store))),
        class: Some(&"top-part".into()),
        ..Default::default()
    })
    .await;
}

async fn list_content(store: &Store<State>) {
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
        let input_classes = ClassList::new(["item-input"]);
        view(ViewProps {
            children: fragment((
                button(ButtonProps {
                    on_press: Some(&mut |_| {
                        let done = { !*handle.done.borrow() };
                        reducers::edit_todo_done(store, id, done);
                    }),
                    class: Some(&done_classes),
                    ..Default::default()
                }),
                text_input(TextInputProps {
                    text: Some(&handle.value.as_observable()),
                    on_blur: Some(&mut |ev| {
                        reducers::edit_todo_value(store, id, ev.get_text());
                    }),
                    class: Some(&input_classes),
                    ..Default::default()
                }),
                button(ButtonProps {
                    on_press: Some(&mut |_ev| reducers::remove_todo(store, id)),
                    class: Some(&"delete-button".into()),
                    ..Default::default()
                }),
            )),
            class: Some(&view_classes),
            ..Default::default()
        })
        .or(async {
            let done_obs = handle.done.as_observable();
            let filter_obs = store.filter.as_observable();
            loop {
                let v = done_obs.get();
                let f = filter_obs.get();
                done_classes.set("done-button-done", v);
                input_classes.set("item-input-done", v);
                let visible = match (f, v) {
                    (DisplayFilter::All, _) => true,
                    (DisplayFilter::Active, false) => true,
                    (DisplayFilter::Complete, true) => true,
                    _ => false,
                };
                view_classes.set("hidden", !visible);
                done_obs.until_change().or(filter_obs.until_change()).await;
            }
        })
        .await;
    }

    let render = &|id| list_item(store, id);
    list(ListProps {
        data: Some(&store.todos_list.as_observable()),
        render: Some(render),
        class: Some(&"list-content".into()),
        ..Default::default()
    })
    .await;
}

async fn bottom_part(store: &Store<State>) {
    async fn active_label(store: &Store<State>) {
        let value = ReactiveCell::new("".into());
        view(ViewProps {
            children: fragment((text(&value.as_observable()),)),
            class: Some(&"active-label-box".into()),
            ..Default::default()
        })
        .or(async {
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
        })
        .await;
    }
    async fn clear_button(store: &Store<State>) {
        let classes = ClassList::new(["clear-button"]);
        button(ButtonProps {
            children: fragment((text(&["Clear Completed"]),)),
            on_press: Some(&mut |_ev| {
                reducers::clear_completed(store);
            }),
            class: Some(&classes),
            ..Default::default()
        })
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
            view(ViewProps {
                children: fragment((radio_button(RadioProps { value: filter }), text(&[label]))),
                class: Some(&classes),
                element_tag: "label",
            })
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
        radio_group(RadioGroupProps {
            children: fragment((view(ViewProps {
                children: buttons,
                class: Some(&"filter-bar".into()),
                ..Default::default()
            }),)),
            value: Some(&store.filter.as_observable()),
            on_change: Some(&mut |filt: DisplayFilter| {
                *store.filter.borrow_mut() = filt;
            }),
        })
        .await;
    }
    let classes = ClassList::new(["bottom-part"]);
    view(ViewProps {
        children: fragment((
            view(ViewProps {
                children: (fragment((active_label(store), clear_button(store)))),
                class: Some(&"bottom-labels".into()),
                ..Default::default()
            }),
            filter_bar(store),
        )),
        class: Some(&classes),
        ..Default::default()
    })
    .or(async {
        loop {
            let hide = store.counts.borrow().total == 0;
            classes.set("hidden", hide);
            store.counts.as_observable().until_change().await;
        }
    })
    .await;
}

async fn footer() {
    view(ViewProps {
        children: fragment((text(&["Made with Async-UI"]),)),
        class: Some(&"footer".into()),
        ..Default::default()
    })
    .await;
}
