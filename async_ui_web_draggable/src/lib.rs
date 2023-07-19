use std::{cell::Cell, future::Future};

use async_ui_web::{
    event_traits::EmitHtmlElementEvent, html::Div, race, shortcut_traits::ShortcutClassList,
    ReactiveCell,
};
use futures_lite::{Stream, StreamExt};

pub struct Dragger<Id: Clone + PartialEq> {
    inner: ReactiveCell<Inner<Id>>,
    horizontal: bool,
    after: Cell<bool>,
}

const PADDINGS_VERTICAL: [&str; 2] = ["padding-top", "padding-bottom"];
const PADDINGS_HORIZONTAL: [&str; 2] = ["padding-left", "padding-right"];
const ZERO_PX: &str = "0px";

#[derive(Clone, Copy)]
enum SpacerType {
    Before,
    None,
    After,
}

impl<Id: Clone + PartialEq> Dragger<Id> {
    pub fn new_vertical() -> Self {
        Self::new(false)
    }
    pub fn new_horizontal() -> Self {
        Self::new(true)
    }
    fn new(horizontal: bool) -> Self {
        Self {
            inner: ReactiveCell::new(Inner::NotDragging),
            horizontal,
            after: Cell::new(false),
        }
    }
    pub async fn wrap_item<F: Future>(&self, id: Id, future: F, wrapper: &Div) {
        let paddings = if self.horizontal {
            &PADDINGS_HORIZONTAL
        } else {
            &PADDINGS_VERTICAL
        };
        wrapper.add_class(style::drag_wrapper);
        wrapper.set_draggable(true);
        let target = &wrapper.element;
        let style = wrapper.style();
        paddings.iter().for_each(|name| {
            let _ = style.set_property(name, ZERO_PX);
        });

        let spacer = Cell::new(SpacerType::None);

        race((
            wrapper.render(async {
                future.await;
            }),
            async {
                let mut dragstart = target.until_dragstart();
                let mut dragend = target.until_dragend();
                loop {
                    let ev = dragstart.next().await.unwrap();
                    let _ = ev.data_transfer().unwrap().set_data("text", "-");
                    *self.inner.borrow_mut() = Inner::DragStarted {
                        from: id.clone(),
                        size: if self.horizontal {
                            target.client_width()
                        } else {
                            target.client_height()
                        } as f64,
                    };
                    dragend.next().await;
                    let mut inner = self.inner.borrow_mut();
                    if matches!(&*inner, Inner::DragStarted { .. }) {
                        *inner = Inner::NotDragging;
                    }
                }
            },
            target.until_dragenter().for_each(|ev| {
                if ev.target().as_ref() == Some(target.as_ref())
                    && matches!(&*self.inner.borrow(), Inner::DragStarted { .. })
                {
                    ev.prevent_default();
                }
            }),
            target.until_dragover().for_each(|ev| {
                if let Inner::DragStarted { from, size } = &*self.inner.borrow() {
                    ev.prevent_default();
                    ev.stop_propagation();
                    if *from == id {
                        return;
                    }
                    let size = *size;
                    let bounding = target.get_bounding_client_rect();
                    let my_start = if self.horizontal {
                        bounding.left()
                    } else {
                        bounding.top()
                    };
                    let dragged_pos = if self.horizontal {
                        ev.client_x()
                    } else {
                        ev.client_y()
                    } as f64;
                    let after = dragged_pos > my_start + size * 0.5;
                    self.after.set(after);
                    match (
                        after,
                        spacer.replace(if after {
                            SpacerType::After
                        } else {
                            SpacerType::Before
                        }),
                    ) {
                        (true, SpacerType::After) | (false, SpacerType::Before) => {}
                        _ => {
                            let after = after as usize;
                            let _ = style.set_property(paddings[after], &format!("{size}px"));
                            let _ = style.set_property(paddings[1 - after], ZERO_PX);
                        }
                    }
                }
            }),
            target.until_dragleave().for_each(|ev| {
                if ev.target().as_ref() == Some(target.as_ref()) {
                    spacer.set(SpacerType::None);
                    paddings.iter().for_each(|name| {
                        let _ = style.set_property(*name, ZERO_PX);
                    });
                }
            }),
            target.until_drop().for_each(|ev| {
                let mut inner = self.inner.borrow_mut();
                *inner = match std::mem::replace(&mut *inner, Inner::NotDragging) {
                    Inner::DragStarted { from, .. } => Inner::Dropped {
                        from,
                        to: id.clone(),
                        after: self.after.get(),
                    },
                    x => x,
                };

                spacer.set(SpacerType::None);
                paddings.iter().for_each(|name| {
                    let _ = style.set_property(*name, ZERO_PX);
                });
                ev.prevent_default();
            }),
        ))
        .await;
    }
    pub fn until_move(&self) -> impl Stream<Item = MoveEvent<Id>> + '_ {
        let uc = self.inner.until_change();
        futures_lite::stream::unfold(uc, move |mut uc| async {
            loop {
                let dragging_from = loop {
                    match &*self.inner.borrow() {
                        Inner::DragStarted { from, .. } => break from.clone(),
                        _ => {}
                    }
                    uc.next().await;
                };
                loop {
                    match &*self.inner.borrow() {
                        Inner::DragStarted { from, .. } if *from == dragging_from => {}
                        Inner::Dropped { from, to, after } => {
                            return Some((
                                MoveEvent {
                                    from: from.clone(),
                                    to: to.clone(),
                                    after: *after,
                                },
                                uc,
                            ))
                        }
                        _ => break,
                    }
                    uc.next().await;
                }
            }
        })
    }
}
pub struct MoveEvent<Id> {
    pub from: Id,
    pub to: Id,
    pub after: bool,
}

enum Inner<Id> {
    NotDragging,
    DragStarted { from: Id, size: f64 },
    Dropped { from: Id, to: Id, after: bool },
}

mod style {
    async_ui_web::css!(
        "
.drag-wrapper {
}
		"
    );
}
