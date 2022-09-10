use std::collections::HashMap;

use async_ui_web::{
    components::{Button, List, ListModel, Text, TextInput, View},
    fragment, mount,
};
use observables::{cell::ObservableCell, Observable, ObservableAsExt};
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
    (View {
        children: fragment![
            TextInput {
                text: &handle.value.as_observable_or_default(),
                on_change_text: &mut |txt| {
                    if let Some(mut value) = handle.value.borrow_mut_opt() {
                        *value = txt;
                    }
                },
                ..Default::default()
            },
            Button {
                children: fragment![Text {
                    text: &handle.done.as_observable_or_default().map(|v| match *v {
                        true => "done",
                        false => "not done",
                    })
                }],
                on_press: &mut |_| {
                    if let Some(mut done) = handle.done.borrow_mut_opt() {
                        *done = !*done;
                    }
                },
                ..Default::default()
            },
            Button {
                children: fragment![Text { text: &"delete" }],
                on_press: &mut |_| {
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
                },
                ..Default::default()
            }
        ],
    })
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
    let value = ObservableCell::new(String::new());
    let submit = || {
        let current_id = {
            let mut bm = store.current_id.borrow_mut();
            bm.0 += 1;
            *bm
        };
        let text = str::to_string(&*value.as_observable().borrow_observable());
        value.borrow_mut().clear();
        store.todos_map.insert(
            current_id,
            Todo {
                value: text,
                done: false,
            },
        );
        store.todos_list.borrow_mut().insert(0, current_id);
    };
    fragment![
        TextInput {
            text: &value.as_observable(),
            on_change_text: &mut |txt| {
                *value.borrow_mut() = txt;
            },
            on_submit: &mut |_txt| {
                submit();
            },
            ..Default::default()
        },
        Button {
            children: fragment![Text { text: &"submit" }],
            on_press: &mut |_ev| {
                submit();
            },
            ..Default::default()
        }
    ]
    .await;
}
