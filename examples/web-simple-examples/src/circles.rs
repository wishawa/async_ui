use std::cell::{Cell, RefCell};

use async_ui_web::{
    components::{Button, Div, Input},
    join,
    prelude_traits::*,
    race, select, DynamicList, NoChild,
};
use futures_lite::StreamExt;
use wasm_bindgen::UnwrapThrowExt;

struct Circle {
    x: i32,
    y: i32,
    scale: f64,
    elem: Div,
}

#[derive(Clone, Copy)]
enum Operation {
    Create { x: i32, y: i32, id: usize },
    Remove { id: usize },
    ChangeScale { which: usize, from: f64, to: f64 },
}

pub async fn circles() {
    let list = DynamicList::new();
    let mut circles = Vec::<Circle>::new();
    let area = Div::new();
    area.add_class(style::area);
    let mut selected = None::<usize>;
    let operations = OperationsBar::new();
    let mut id_counter = 0;

    Div::new()
    .render(join((
		"click to add a circle. right click to adjust diameter.".render(),
		operations.render(),
        area.render(join((
            list.render(),
			async {
                let mut mousemove_stream = area.until_mousemove();
				loop {
					let mut mosue_to_track = None;
					let mut op_to_do = None;
					select!(
						ev = mousemove_stream.next() => {
							mosue_to_track = ev;
						}
						ev = area.until_contextmenu() => {
							ev.prevent_default();
							if let Some(selected) = selected {
								let (mx, my) = (ev.offset_x(), ev.offset_y());
								let circle = &circles[selected];
								let old_size = circle.scale;
								if let Some(new_size) = context_menu(&circle.elem, old_size, mx, my).await {
									let op = Operation::ChangeScale { which: selected, from: old_size, to: new_size };
									operations.add_operation(op);
									op_to_do = Some(op);
								}
							}
						}
						ev = area.until_click() => {
							let (mx, my) = (ev.offset_x(), ev.offset_y());
							let op = Operation::Create { x: mx, y: my, id: id_counter };
                            id_counter += 1;
							operations.add_operation(op);
							op_to_do = Some(op);
							mosue_to_track = Some(ev);
						}
						op = operations.until_do() => {
							op_to_do = Some(op);
						}
					);

					match op_to_do {
						Some(Operation::ChangeScale { which,  to, .. }) => {
							let circle = &mut circles[which];
							circle.scale = to;
							circle.elem.style().set_property(CSS_CIRCLE_SCALE, &to.to_string()).unwrap_throw();
						}
						Some(Operation::Create { x, y, id }) => {
							let elem = Div::new();
							elem.add_class(style::circle);
							elem.style().set_property("left", &format!("{x}px")).unwrap_throw();
							elem.style().set_property("top", &format!("{y}px")).unwrap_throw();
                            list.insert(id, elem.render(NoChild), None);
							circles.push(Circle {x, y, scale: 1.0, elem});
						}
						Some(Operation::Remove {id}) => {
							circles.pop().unwrap();
							list.remove(&id);
						}
						None => {}
					}

					if let Some(ev) = mosue_to_track {
						let (mx, my) = (ev.offset_x(), ev.offset_y());
						let new_selected = circles
							.iter()
							.enumerate()
							.filter_map(|(idx, Circle { x, y, scale, elem })| {
								let dist = (((x - mx).pow(2) + (y - my).pow(2)) as f64).sqrt();
								(dist < *scale * 25.0).then_some((idx, dist, elem))
							})
							.min_by(|(_, xdist, _), (_, ydist, _)| xdist.total_cmp(ydist))
							.map(|(idx, _, _)| idx);
						match (selected, new_selected) {
							(Some(old_idx), Some(new_idx)) if old_idx == new_idx => {}
							(Some(old_idx), _) => {
								if let Some(c) = circles.get(old_idx) {
									c.elem.del_class(style::circle_selected);
								}
							}
							_ => {}
						}
						if let Some(new_idx) = new_selected {
							circles[new_idx].elem.add_class(style::circle_selected);
						}
						selected = new_selected;
					}
				}
			}
        ))),
    )))
    .await;
}

/// Show a context menu at the specified x,y position.
/// The `elem` should be the circle whose size we will adjust.
/// The `initial_size` is the current size of that circle.
/// Returns the new size if the user dragged the slider.
async fn context_menu(elem: &Div, initial_size: f64, x: i32, y: i32) -> Option<f64> {
    let backdrop = Div::new();
    backdrop.add_class(style::backdrop);
    let container = Div::new();
    container.add_class(style::popup);
    container
        .style()
        .set_property("left", &format!("{x}px"))
        .unwrap_throw();
    container
        .style()
        .set_property("top", &format!("{y}px"))
        .unwrap_throw();
    let context_menu = Div::new();
    let adjust_button = Button::new();
    let mut final_size = None;
    race((
        backdrop.render(container.render(async {
            race((
                context_menu.render(adjust_button.render("Adjust Diameter".render())),
                async {
                    adjust_button.until_click().await.stop_propagation();
                },
            ))
            .await;
            let slider = Input::new();
            slider.set_type("range");
            slider.set_min("0.1");
            slider.set_max("10.0");
            slider.set_step("0.01");
            slider.set_value(&initial_size.to_string());
            join((slider.render(), async {
                loop {
                    slider.until_input().await;
                    let value = slider.value_as_number();
                    elem.style()
                        .set_property(CSS_CIRCLE_SCALE, &value.to_string())
                        .unwrap_throw();
                    final_size = Some(value);
                }
            }))
            .await;
        })),
        async {
            loop {
                container.until_click().await.stop_propagation();
            }
        },
        async {
            backdrop.until_click().await.stop_propagation();
        },
    ))
    .await;
    final_size
}

struct OperationsBar {
    user_ops: RefCell<Vec<Operation>>,
    cursor: Cell<usize>,
    undo_btn: Button,
    redo_btn: Button,
}
impl OperationsBar {
    fn new() -> Self {
        let redo_btn = Button::new();
        redo_btn.set_disabled(true);
        let undo_btn = Button::new();
        undo_btn.set_disabled(true);
        Self {
            redo_btn,
            undo_btn,
            user_ops: Default::default(),
            cursor: Default::default(),
        }
    }
    async fn render(&self) {
        {
            let top = Div::new();
            top.add_class(style::top_part);
            top
        }
        .render(join((
            self.undo_btn.render("Undo".render()),
            self.redo_btn.render("Redo".render()),
        )))
        .await;
    }
    async fn until_do(&self) -> Operation {
        select!(
            _ = self.undo_btn.until_click() => {
                let last_idx = self.cursor.get() - 1;
                self.cursor.set(last_idx);
                self.redo_btn.set_disabled(false);
                self.undo_btn.set_disabled(last_idx == 0);
                match self.user_ops.borrow()[last_idx] {
                    Operation::ChangeScale { which, from, to } => Operation::ChangeScale { which, from: to, to: from },
                    Operation::Create {id, ..} => Operation::Remove { id},
                    Operation::Remove {..} => unreachable!()
                }
            },
            _ = self.redo_btn.until_click() => {
                let last_idx = self.cursor.get();
                self.cursor.set(last_idx + 1);
                let user_ops = self.user_ops.borrow();
                self.undo_btn.set_disabled(false);
                self.redo_btn.set_disabled(last_idx + 1 == user_ops.len());
                user_ops[last_idx]
            }
        )
    }
    fn add_operation(&self, op: Operation) {
        let last_idx = self.cursor.get();
        let mut user_ops = self.user_ops.borrow_mut();
        user_ops.truncate(last_idx);
        user_ops.push(op);
        self.cursor.set(last_idx + 1);
        self.undo_btn.set_disabled(false);
        self.redo_btn.set_disabled(true);
    }
}

const CSS_CIRCLE_SCALE: &str = "--circle-scale";
mod style {
    use async_ui_web::css;

    css!(
        "
.top-part {
	display: flex;
	flex-direction: row;
}
.area {
	position: relative;
	height: 400px;
	width: 600px;
	border: 1px solid black;
	overflow: hidden;
}
.circle {
	position: absolute;
	pointer-events: none;
	height: 48px;
	width: 48px;
	border-radius: 25px;
	border: 1px solid black;
	--circle-scale: 1.0;
	transform: translate(-50%, -50%) scale(var(--circle-scale));
	z-index: 50;
}
.circle-selected {
	background-color: rgba(0,0,0,0.2);
}
.backdrop {
	position: absolute;
	left: 0;
	right: 0;
	top: 0;
	bottom: 0;
}
.popup {
	position: absolute;
	z-index: 100;
	background-color: white;
	box-shadow: 0 0 5px 0 black;
	border-radius: 4px;
}
.popup > input {
	margin: 4px;
}
		"
    );
}
