use std::{fmt::Debug, future::pending, pin::Pin};

use async_ui_web::{
    event_traits::EmitElementEvent,
    html::{Anchor, Button, Div, Span, H3},
    join,
    shortcut_traits::{ShortcutClassList, ShortcutClassListBuilder, ShortcutRenderStr},
    ReactiveCell,
};
use futures_lite::{Future, StreamExt};
use wasm_bindgen::prelude::wasm_bindgen;
use x_bow::{IntoPath, Path, PathExt, Store, Trackable};

#[derive(Trackable)]
#[track(deep)]
struct HasTwoFields<T1, T2> {
    field_1: T1,
    field_2: T2,
}

#[derive(Trackable)]
#[track(deep)]
struct HasVec<T>(Vec<T>);

#[derive(Trackable)]
#[track(deep)]
struct HasOneField<T> {
    field: T,
}

struct GlobalState {
    version: ReactiveCell<u64>,
}

trait CanDisplay: Trackable {
    fn render<'p, P: Path<Out = Self> + Clone + Debug + 'p>(
        path: P,
        global: &'p GlobalState,
    ) -> Pin<Box<dyn Future<Output = ()> + 'p>>;
}

impl<T1: Trackable + CanDisplay, T2: Trackable + CanDisplay> CanDisplay for HasTwoFields<T1, T2> {
    fn render<'p, P: Path<Out = Self> + Clone + Debug + 'p>(
        path: P,
        global: &'p GlobalState,
    ) -> Pin<Box<dyn Future<Output = ()> + 'p>> {
        Box::pin(async move {
            one_wrapper(
                &path,
                join((
                    <T1 as CanDisplay>::render(
                        path.clone().build_path().field_1().into_path(),
                        global,
                    ),
                    <T2 as CanDisplay>::render(
                        path.clone().build_path().field_2().into_path(),
                        global,
                    ),
                )),
                global,
            )
            .await;
        })
    }
}
impl<T: Trackable + CanDisplay> CanDisplay for HasOneField<T> {
    fn render<'p, P: Path<Out = Self> + Clone + Debug + 'p>(
        path: P,
        global: &'p GlobalState,
    ) -> Pin<Box<dyn Future<Output = ()> + 'p>> {
        Box::pin(async move {
            one_wrapper(
                &path,
                <T as CanDisplay>::render(path.clone().build_path().field().into_path(), global),
                global,
            )
            .await;
        })
    }
}
impl<T: Trackable + CanDisplay> CanDisplay for HasVec<T> {
    fn render<'p, P: Path<Out = Self> + Clone + Debug + 'p>(
        path: P,
        global: &'p GlobalState,
    ) -> Pin<Box<dyn Future<Output = ()> + 'p>> {
        Box::pin(async move {
            one_wrapper(
                &path,
                horizontal_wrapper(join({
                    let futures = path
                        .borrow_opt()
                        .unwrap()
                        .0
                        .iter()
                        .enumerate()
                        .map(|(idx, _)| {
                            <T as CanDisplay>::render(
                                path.clone().build_path().t0().index(idx).into_path(),
                                global,
                            )
                        })
                        .collect::<Vec<_>>();
                    futures
                })),
                global,
            )
            .await;
        })
    }
}

async fn horizontal_wrapper<F: Future>(f: F) -> F::Output {
    Div::new().with_class(style::horizontal).render(f).await
}

#[derive(Trackable)]
struct Leaf<T>(T);

impl<T: Trackable + Debug> CanDisplay for Leaf<T> {
    fn render<'p, P: Path<Out = Self> + Clone + Debug + 'p>(
        path: P,
        global: &'p GlobalState,
    ) -> Pin<Box<dyn Future<Output = ()> + 'p>> {
        Box::pin(async move {
            one_wrapper(&path, pending::<()>(), global).await;
        })
    }
}

#[wasm_bindgen(start)]
#[cfg(feature = "csr")]
pub fn run() {
    use async_ui_web::mount;

    console_error_panic_hook::set_once();
    mount(app());
}

async fn app() {
    let data = HasTwoFields {
        field_1: HasTwoFields {
            field_1: Leaf("hello"),
            field_2: HasOneField {
                field: Leaf("world"),
            },
        },
        field_2: HasVec((0..4).map(|i| HasOneField { field: Leaf(i) }).collect()),
    };
    join((display_root(data), instructions())).await;
}
async fn display_root<T: Trackable + CanDisplay>(data: T) {
    let store = Store::new(data);
    let global_state = GlobalState {
        version: ReactiveCell::new(1),
    };
    let clear = Button::new();
    join((
        T::render(store.build_path().into_path(), &global_state),
        clear.render("Clear".render()),
        clear.until_click().for_each(|_| {
            *global_state.version.borrow_mut() += 1;
        }),
    ))
    .await;
}

async fn one_wrapper<T: Trackable, F: Future>(
    state: &(impl Path<Out = T> + Debug),
    inner: F,
    global: &GlobalState,
) {
    let wrapper = Div::new();
    wrapper.set_title("click me!");
    let mut prev_ver = 0;
    join((
        wrapper
            .with_class(style::node_box)
            .render(join((format!("{state:?}").render(), inner))),
        state
            .until_change()
            .map(|_| 1)
            .race(state.until_bubbling_change().map(|_| 2))
            .race(global.version.until_change().map(|_| 3))
            .for_each(|ch| {
                if std::mem::replace(&mut prev_ver, *global.version.borrow()) != prev_ver {
                    wrapper.del_classes([style::changed_regular, style::changed_bubbling]);
                }
                match ch {
                    1 => wrapper.add_class(style::changed_regular),
                    2 => wrapper.add_class(style::changed_bubbling),
                    _ => {}
                }
            }),
        wrapper.until_click().for_each(|ev| {
            ev.stop_propagation();
            *global.version.borrow_mut() += 1;
            state.notify_changed();
        }),
    ))
    .await;
}

async fn instructions() {
    Div::new()
        .with_class(style::instruction)
        .render(join((
            H3::new().render("Help".render()),
            Span::new().render(join((
                "Click a box to ".render(),
                link("borrow_mut", "https://docs.rs/x_bow/latest/x_bow/trait.PathExt.html#method.borrow_opt_mut"),
                " that piece of data.".render(),
            ))),
            Div::new()
                .with_classes([
                    style::node_box,
                    style::instruction_node_box,
                    style::changed_bubbling,
                ])
                .render(join((
                    "Boxes that turn green have their ".render(),
                    link("until_change_bubbling", "https://docs.rs/x_bow/latest/x_bow/trait.PathExt.html#method.until_bubbling_change"),
                    " listener notified.".render(),
                ))),
            Div::new()
                .with_classes([
                    style::node_box,
                    style::instruction_node_box,
                    style::changed_regular,
                ])
                .render(join((
                    "Boxes that turn red have their ".render(),
                    link("until_change", "https://docs.rs/x_bow/latest/x_bow/trait.PathExt.html#method.until_change"),
                    " listener notified.".render(),
                ))),
            Div::new()
                .with_classes([
                    style::node_box,
                    style::instruction_node_box,
                    style::changed_bubbling,
                    style::changed_regular,
                ])
                .render(join((
                    "Boxes that turn yellow have both their listeners notified".render(),
                ))),
        )))
        .await;
}

async fn link(text: &str, url: &str) {
    {
        let a = Anchor::new();
        a.set_href(url);
        a
    }
    .render(text.render())
    .await;
}

mod style {
    async_ui_web::css!(
        r#"
.node-box {
    border: 3px solid rgba(0,0,0,0.1);
    padding: 0.5rem;
    margin: 1.0rem;
    border-radius: 0.5rem;
    cursor: pointer;
}

.instruction {
    border-block-start: 1px solid black;
    margin-block-start: 2rem;
}
.instruction-node-box.node-box {
    cursor: default;
}

.horizontal {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
}

.changed-regular {
    border-color: red;
    border-style: dotted;
}
.changed-bubbling {
    border-color: green;
    border-style: dashed;
}
.changed-bubbling.changed-regular {
    border-color: orange;
    border-style: solid;
}
        "#
    );
}
