use crate::lifetimed::Lifetimed;

pub mod dedupe;
pub mod map;
pub mod source;

#[derive(Clone, Copy)]
pub enum PushMode {
	Requested,
	NotRequested
}

pub trait Pushable<L>
where
	L: Lifetimed,
{
	fn push<'s, 'v>(&'s self, value: L::Value<'v>, mode: PushMode)
	where
		Self: 'v;
}

pub unsafe trait Signal<L>
where
	L: Lifetimed,
{
	fn add_listener<'s>(&'s self, listener: *const dyn Pushable<L>);
	fn remove_listener<'s>(&'s self, listener: *const dyn Pushable<L>);
	fn request_fire<'s>(&'s self, listener: *const dyn Pushable<L>);
}
