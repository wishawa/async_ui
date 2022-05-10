use std::cell::RefCell;

use crate::{
	sub::{ParentSub, SubManager},
	Visitable,
};

pub struct SignalCached<'p, C> {
	parent: &'p dyn for<'k> Visitable<dyn FnMut(C) + 'k>,
	value: RefCell<Option<C>>,
	parent_sub: ParentSub<'p>,
	my_sub: SubManager,
}

impl<'p, C> SignalCached<'p, C> {
	pub fn new(parent: &'p dyn for<'k> Visitable<dyn for<'x> FnMut(C) + 'k>) -> Self {
		Self {
			parent,
			parent_sub: parent.get_sub(),
			value: RefCell::new(None),
			my_sub: SubManager::new(),
		}
	}
}

impl<'a, 'p, C, V> Visitable<V> for SignalCached<'p, C>
where
	C: 'p,
	V: ?Sized + for<'x> FnMut(&'x C) + 'a,
{
	fn visit<'v, 's>(&'s self, visitor: &'v mut V)
	where
		Self: 'v,
	{
		let mut opt = self.value.borrow_mut();
		if opt.is_none() {
			self.parent.visit(&mut |inp| {
				*opt = Some(inp);
			});
		}
		visitor(opt.as_ref().unwrap());
	}
	fn get_sub<'s>(&'s self) -> ParentSub<'s> {
		(&self.my_sub).into()
	}
}
