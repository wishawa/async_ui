use std::collections::HashMap;

use async_ui_web::{
    components::{Button, List, ListModel, Text, TextInput, View},
    fragment, mount,
};
use observables::{cell::ObservableCell, Observable, ObservableExt};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use x_bow::{create_store, Store, Track};

#[derive(Track)]
struct State {
    todos_map: HashMap<TodoId, Todo>,
    #[x_bow(no_track)]
    todos_list: ListModel<TodoId>,
    current_id: TodoId,
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
    let text = &handle.value.to_observable_or_default();
    let on_press_delete = &|_ev| {
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
    };
    let done = handle.done.to_observable_or_default();
    let done_text = done.map(|v| match *v {
        true => "done",
        false => "not done",
    });
    let on_press_toggle = &|_ev| {
        if let Some(mut done) = handle.done.borrow_mut_opt() {
            *done = !*done;
        }
    };
    (View {
        children: fragment![
            Text { text },
            Button {
                children: fragment![Text { text: &done_text }],
                on_press: on_press_toggle,
                ..Default::default()
            },
            Button {
                children: fragment![Text { text: &"delete" }],
                on_press: on_press_delete,
                ..Default::default()
            }
        ],
    })
    .await
}
async fn list_content(store: &Store<State>) {
    let render = &|id| list_item(store, id);
    (List {
        data: &store.todos_list.to_observable(),
        render,
    })
    .await;
}
async fn input_box(store: &Store<State>) {
    let value = ObservableCell::new("".into());
    fragment![
        TextInput {
            text: &value.as_observable(),
            on_input: &|ev| {
                if let Some(v) = ev.data() {
                    *value.borrow_mut() = v;
                }
            }
        },
        Button {
            children: fragment![Text { text: &"submit" }],
            on_press: &|_ev| {
                let current_id = {
                    let mut bm = store.current_id.borrow_mut();
                    bm.0 += 1;
                    *bm
                };
                let value = str::to_string(&*value.as_observable().observable_borrow());
                store
                    .todos_map
                    .insert(current_id, Todo { value, done: false });
                store.todos_list.borrow_mut().insert(0, current_id);
            },
            ..Default::default()
        }
    ]
    .await
}
