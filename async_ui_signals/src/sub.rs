use std::{cell::{RefCell, Cell}, collections::{BTreeMap}, task::Waker};

use self::{leaf::{LeafAddress}, version::Version};

mod leaf {
	use std::pin::Pin;
	enum Untyped {}
	#[derive(PartialEq, Eq, PartialOrd, Ord)]
	pub struct LeafAddress(*const Untyped);
	impl LeafAddress {
		pub fn new<T>(address: Pin<&T>) -> Self {
			Self (address.get_ref() as *const T as *const Untyped)
		}
	}
}

mod version {
    use std::cell::Cell;

	pub struct Version(Cell<usize>);

	impl Version {
		pub fn new() -> Self {
			Self(Cell::new(0))
		}
		pub fn increment(&self) {
			self.0.set(self.0.get() + 1)
		}
		pub fn compare_update(&self, other: &Self) -> bool {
			if other.0 > self.0 {
				self.0.set(other.0.get());
				true
			}
			else {
				false
			}
		}
	}
}

pub struct SubManager {
    leaves: RefCell<BTreeMap<LeafAddress, Waker>>,
	version: Version
}

impl SubManager {
	pub fn new() -> Self {
		Self { leaves: Default::default(), version: Version::new() }
	}
	pub fn add_leaf(&self, address: LeafAddress, waker: Waker) {
		self.leaves.borrow_mut().insert(address, waker);
	}
	pub fn remove_leaf(&self, address: &LeafAddress) {
		self.leaves.borrow_mut().remove(address);
	}
	pub fn increment_version(&self) {
		self.version.increment();
	}
}

#[derive(Clone, Copy)]
pub struct ParentSub<'a>(&'a SubManager);

impl<'a> From<&'a SubManager> for ParentSub<'a> {
	fn from(s: &'a SubManager) -> Self {
		Self(s)
	}
}

impl<'a> ParentSub<'a> {
	pub fn compare_update_version(&self, my_version: &Version) -> bool {
		my_version.compare_update(&self.0.version)
	}
}