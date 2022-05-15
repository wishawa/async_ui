use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
};

use crate::{
	ext::UnsizeSignal,
	lifetimed::{Borrowed, LifetimedCovariant, Lifetimed},
};

use super::{
	map::{map, MapTo},
	Signal,
};

pub fn dedupe<'p, S, L, C>(signal: &'p S) -> impl Signal<Borrowed<C>> + 'p
where
	C: PartialEq + 'p,
	L: Lifetimed + 'p,
	S: ?Sized + UnsizeSignal<L>,
	for<'x> L::Value<'x>: Into<C>,
{
	let mut state: Option<C> = None;
	let mapper = move |inp: L::Value<'_>, next: MapTo<'_, Borrowed<C>>| {
		let converted: C = inp.into();
		match state.as_ref() {
			Some(old) if old == &converted => {}
			_ => {
				next.push(&converted);
				state = Some(converted);
			}
		}
	};
	map(signal, mapper)
}

pub fn dedupe_by_hash<'p, S, L>(signal: &'p S) -> impl Signal<L> + 'p
where
	L: LifetimedCovariant + 'p,
	S: ?Sized + UnsizeSignal<L>,
	for<'x> L::Value<'x>: Hash,
{
	let mut last_hash: Option<u64> = None;
	let mapper = move |inp: L::Value<'_>, next: MapTo<'_, L>| {
		let mut hasher = DefaultHasher::new();
		inp.hash(&mut hasher);
		let hash = hasher.finish();
		match last_hash {
			Some(old) if old == hash => {}
			_ => {
				next.push(L::shorten(inp));
				last_hash = Some(hash);
			}
		}
	};
	map(signal, mapper)
}
