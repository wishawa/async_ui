#![feature(generic_associated_types)]

use std::{
	cell::Cell,
	future::Future,
	marker::PhantomData,
	ops::{Index, Range, RangeFrom},
	rc::Rc,
	task::Poll,
};

use async_ui_reactive::local::Rx;
use async_ui_signals::{
	mapper::Mapper,
	nodes::{for_each::SignalForEach, map::SignalMap, source::SignalSource},
	Signal,
};
use async_ui_utils::Join;
use async_ui_web::{create_portal, hidable, list, mount, Render};
use async_ui_web_html::{anchor, button, div, span, text};
use wasm_bindgen::{
	prelude::{wasm_bindgen, Closure},
	JsCast, JsValue,
};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
	use std::panic;
	panic::set_hook(Box::new(console_error_panic_hook::hook));
	mount((my_component(),));
	Ok(())
}

async fn my_component() {
	let (p_ent, p_ext) = create_portal();
	#[rustfmt::skip]
    Render::from((
        signal_test(),
        // div().children((text().content("hello world"),)),
        // counter(&5),
        // div().children((text().content("hi"), p_ext.render(), text().content("bye"))),
        // p_ent.render((
        //     text().content("oh my"),
        //     text().content("confusion"),
        //     anchor()
        //         .href("https://example.com")
        //         .on_click(|ev| ev.prevent_default())
        //         .children((text().content("qwerqwer"),)),
        // )),
        // list_test(),
        // take_children((hidable_test(),)),
    ))
    .await;
}
async fn hidable_test() {
	let switch = Rx::new(true);
	#[rustfmt::skip]
    Join::from((
        hidable(&switch, (text().content("i may be hidden"),)),
        async {
            Timeout::new(1000).await;
            *switch.borrow_mut() = false;
            Timeout::new(1000).await;
            *switch.borrow_mut() = true;
        },
    ))
    .await;
}
async fn counter(step: &i32) {
	let value = Rx::new(0);
	let content = Rx::new("0".into());
	#[rustfmt::skip]
    Render::from((
        button()
            .on_click(|_ev| {
                value.visit_mut(|m| *m -= *step);
                content.replace(value.get().to_string());
            })
            .children((text().content("-"),)),
        span().children((text().content_reactive(&content),)),
        button()
            .on_click(|_ev| {
                value.visit_mut(|m| *m += *step);
                content.replace(value.get().to_string());
            })
            .children((text().content("+"),)),
    ))
    .await;
}

async fn signal_test() {
	let source = String::from("hellow world");
	let mut source = SignalSource::new(source);

	struct SliceMapper<T: ?Sized + Index<RangeFrom<usize>>>(RangeFrom<usize>, PhantomData<T>);
	impl<T: ?Sized + Index<RangeFrom<usize>>> Mapper for SliceMapper<T> {
		type Input<'i> = &'i T where Self: 'i;
		type Output<'o> = &'o T::Output where Self: 'o;
		fn map<'x>(&self, input: Self::Input<'x>) -> Self::Output<'x> {
			&input[self.0.clone()]
		}
	}

	let mapper: SliceMapper<String> = SliceMapper(1.., PhantomData);

	let mapped = SignalMap::new(mapper, &source);

	let mapper2: SliceMapper<str> = SliceMapper(2.., PhantomData);
	let mapped2 = SignalMap::new(mapper2, &mapped);
	// fn take_sig(signal: &(dyn for<'k> Listenable<dyn for<'i> Pushable<&'i str> + 'k> + '_)) {
	// let mapper2: SliceMapper<str> = SliceMapper(1..3, PhantomData);
	// let mapped2 = SignalMap::new(mapper2, signal);
	// }
	struct TestMapper {
		node: web_sys::Text,
	};
	impl Mapper for TestMapper {
		type Input<'i> = &'i str where Self: 'i;
		type Output<'o> = () where Self :'o;
		fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
		where
			Self: 'm,
		{
			self.node.set_data(input);
		}
	}

	async fn take_sig<'a>(signal: &'a (dyn for<'i> Signal<SignalForEach<'i, TestMapper>> + 'a)) {
		let text = text().content("hi");
		let tr = text.get_reference();
		let tm = TestMapper { node: tr };
		Join::from((SignalForEach::new(signal, tm), Render::from((text,)))).await;
	}
	Join::from((take_sig(&mapped2), async {
		loop {
			Timeout::new(2000).await;
			source.visit_mut(|s| {
				s.replace_range(0..6, "hola555");
			});
		}
	}))
	.await;
}

// async fn sooidsfjasoife() {
//     loop {
//         play_game().await;
//         show_game_over().await;
//     }
// }

async fn list_test() {
	let children = Rx::new(vec![]);
	let child_factory = |key: &i32| Render::from((text().content(&key.to_string()),));
	Join::from((list(&children, child_factory), async {
		for i in 0..20 {
			Timeout::new(1000).await;
			children.borrow_mut().push(i);
		}
	}))
	.await;
}

async fn take_children(children: impl Into<Render<'_>>) {
	Render::from((div().children((
		text().content("below is my children"),
		div().children((children.into(),)),
	)),))
	.await
}

struct Timeout {
	inner: TimeoutInner,
}
impl Timeout {
	pub fn new(duration: u32) -> Self {
		Self {
			inner: TimeoutInner::Duration(duration),
		}
	}
}
enum TimeoutInner {
	Duration(u32),
	Fired(Rc<Cell<bool>>),
	Null,
}
impl Future for Timeout {
	type Output = ();

	fn poll(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Self::Output> {
		self.inner = match std::mem::replace(&mut self.inner, TimeoutInner::Null) {
			TimeoutInner::Duration(d) => {
				let fired = Rc::new(Cell::new(false));
				let fired_copy = fired.clone();
				let waker = cx.waker().to_owned();
				let clos = Closure::once_into_js(move || {
					fired_copy.set(true);
					waker.wake()
				});
				web_sys::window()
					.unwrap()
					.set_timeout_with_callback_and_timeout_and_arguments_0(
						&clos.as_ref().unchecked_ref(),
						d as i32,
					)
					.expect("setTimeout failed");
				TimeoutInner::Fired(fired)
			}
			TimeoutInner::Fired(f) => {
				if f.get() {
					return Poll::Ready(());
				}
				TimeoutInner::Fired(f)
			}
			null => null,
		};
		Poll::Pending
	}
}
