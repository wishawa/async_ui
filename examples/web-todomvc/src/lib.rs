use std::collections::HashMap;

use async_ui_web::{
    components::{
        button, text, text_input, view, ButtonProp, List, ListModel, TextInputProp, ViewProp,
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
    #[x_bow(no_track)]
    todos_list: ListModel<TodoId>,
    current_id: TodoId,
}

mod reducers {
    use x_bow::Store;

    use crate::{State, Todo, TodoId};

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
    });
    fragment![input_box(&store), list_content(&store)].await
}
async fn list_item(store: &Store<State>, id: TodoId) {
    let handle = store.todos_map.handle_at(id);
    let done_classes = ClassList::new(["done-button"]);
    view([ViewProp::Children(fragment((
        text_input([
            TextInputProp::Text(&handle.value.as_observable_or_default()),
            TextInputProp::OnBlur(&mut |ev| {
                if let Some(mut value) = handle.value.borrow_mut_opt() {
                    *value = ev.get_text();
                }
            }),
        ]),
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
        button([
            ButtonProp::Children(fragment((text(&"delete"),))),
            ButtonProp::OnPress(&mut |_ev| reducers::remove_todo(store, id)),
        ]),
        async {
            let done_obs = handle.done.as_observable_or_default();
            loop {
                done_classes.set("done-button-done", *done_obs.borrow_observable());
                done_obs.until_change().await;
            }
        },
    )))])
    .await;
}
async fn list_content(store: &Store<State>) {
    let render = &|id| list_item(store, id);
    (List {
        data: &store.todos_list.as_observable(),
        render,
    })
    .await;
}
async fn input_box(store: &Store<State>) {
    let value = ReactiveCell::new(String::new());
    let submit = || {
        let text = value.as_observable().borrow_observable().clone();
        value.borrow_mut().clear();
        reducers::add_todo(store, text);
    };
    fragment((
        text_input([
            TextInputProp::Text(&value.as_observable()),
            TextInputProp::OnSubmit(&mut |ev| {
                *value.borrow_mut() = ev.get_text();
                submit();
            }),
            TextInputProp::OnBlur(&mut |ev| {
                *value.borrow_mut() = ev.get_text();
            }),
        ]),
        button([
            ButtonProp::Children(fragment((text(&"submit"),))),
            ButtonProp::OnPress(&mut |_ev| {
                submit();
            }),
        ]),
    ))
    .await;
}
