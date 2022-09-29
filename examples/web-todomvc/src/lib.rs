use std::collections::HashMap;

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
    todos_map: HashMap<TodoId, Todo>,
    current_id: TodoId,
    #[x_bow(no_track)]
    todos_list: ListModel<TodoId>,
    #[x_bow(no_track)]
    filter: DisplayFilter,
}

#[derive(Clone, Copy, PartialEq)]
enum DisplayFilter {
    All,
    Active,
    Complete,
}

mod reducers {
    use crate::{State, Todo, TodoId};
    use x_bow::Store;

    pub(super) fn get_id_incremented(store: &Store<State>) -> TodoId {
        let mut bm = store.current_id.borrow_mut();
        bm.0 += 1;
        *bm
    }

    pub(super) fn add_todo(store: &Store<State>, text: String) {
        let id = get_id_incremented(store);
        store.todos_map.insert(
            id,
            Todo {
                value: text,
                done: false,
            },
        );
        store.todos_list.borrow_mut().insert(0, id);
    }

    pub(super) fn remove_todo(store: &Store<State>, id: TodoId) {
        store.todos_map.remove(&id);
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
    pub(super) fn set_all(store: &Store<State>, completed: bool) {
        for key in store.todos_list.borrow().underlying_vector().iter() {
            if let Some(mut done) = store
                .todos_map
                .handle_at(key.to_owned())
                .done
                .borrow_mut_opt()
            {
                *done = completed;
            }
        }
    }
    pub(super) fn clear_completed(store: &Store<State>) {
        let mut to_remove = Vec::new();
        let mut bm = store.todos_list.borrow_mut();
        for (idx, key) in bm.underlying_vector().iter().enumerate() {
            if let Some(done) = store.todos_map.handle_at(key.to_owned()).done.borrow_opt() {
                if *done {
                    to_remove.push((idx, key.to_owned()));
                }
            }
        }
        for (idx, key) in to_remove.into_iter().rev() {
            store.todos_map.remove(&key);
            bm.remove(idx);
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
    });
    view([
        ViewProp::Children(fragment((view([
            ViewProp::Children(fragment((
                add_input_box(&store),
                list_content(&store),
                bottom_part(&store),
            ))),
            ViewProp::Class(&ClassList::new(["main-container"])),
        ]),))),
        ViewProp::Class(&ClassList::new(["wrapper"])),
    ])
    .await;
}
async fn bottom_part(store: &Store<State>) {
    let classes = ClassList::new(["bottom-part"]);
    use async_ui_web::futures_lite::FutureExt;
    view([
        ViewProp::Children(fragment((filter_bar(store),))),
        ViewProp::Class(&classes),
    ])
    .or(async {
        loop {
            let hide = store.todos_list.borrow().len() == 0;
            classes.set("hidden", hide);
            store.todos_list.as_observable().until_change().await;
        }
    })
    .await;
}
async fn filter_bar(store: &Store<State>) {
    async fn filter_button(store: &Store<State>, filter: DisplayFilter) {
        use async_ui_web::futures_lite::FutureExt;
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
        ViewProp::Class(&ClassList::new(["filter-bar"])),
    ])
    .await;
}
async fn list_item(store: &Store<State>, id: TodoId) {
    let handle = store.todos_map.handle_at(id);
    let done_classes = ClassList::new(["done-button"]);
    let view_classes = ClassList::new(["list-item"]);
    view([
        ViewProp::Children(fragment((
            button([
                ButtonProp::Children(fragment((text(
                    &handle.done.as_observable_or_default().map(|v| match *v {
                        true => "done",
                        false => "not done",
                    }),
                ),))),
                ButtonProp::OnPress(&mut |_| {
                    if let Some(mut done) = handle.done.borrow_mut_opt() {
                        *done = !*done;
                    }
                }),
                ButtonProp::Class(&done_classes),
            ]),
            text_input([
                TextInputProp::Text(&handle.value.as_observable_or_default()),
                TextInputProp::OnBlur(&mut |ev| {
                    if let Some(mut value) = handle.value.borrow_mut_opt() {
                        *value = ev.get_text();
                    }
                }),
                TextInputProp::Class(&ClassList::new(["item-input"])),
            ]),
            button([
                ButtonProp::Children(fragment((text(&"delete"),))),
                ButtonProp::OnPress(&mut |_ev| reducers::remove_todo(store, id)),
            ]),
            async {
                let done_obs = handle.done.as_observable_or_default();
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
                    use async_ui_web::futures_lite::FutureExt;
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
        ListProp::Class(&ClassList::new(["list-content"])),
    ])
    .await;
}
async fn top_part(store: &Store<State>) {}
async fn toggle_all_button(store: &Store<State>) {
    button([
        ButtonProp::Class(&ClassList::new(["toggle-all-button"])),
        ButtonProp::OnPress(&mut |_ev| {
            // reducers::
        }),
    ])
    .await;
}
async fn add_input_box(store: &Store<State>) {
    let value = ReactiveCell::new(String::new());
    view([
        ViewProp::Children(fragment((text_input([
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
            TextInputProp::Class(&ClassList::new(["add-input"])),
            TextInputProp::Placeholder(&"What needs to be done?"),
        ]),))),
        ViewProp::Class(&ClassList::new(["add-box"])),
    ])
    .await;
}
