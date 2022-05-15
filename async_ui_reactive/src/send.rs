mod channel;
mod rx;
pub use channel::*;
use std::sync::{
	atomic::AtomicUsize, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError,
};

use self::rx::{RxGuard, RxGuardMut, RxGuardMutBase, RxGuardMutSilent};

use super::subscriptions::Subscriptions;

pub struct Rx<T> {
	data: RwLock<T>,
	subscriptions: Mutex<Subscriptions>,
	version: AtomicUsize,
}

type RefRead<'a, T> = RwLockReadGuard<'a, T>;
type RefWrite<'a, T> = RwLockWriteGuard<'a, T>;

impl<T> Rx<T> {
	pub fn new(value: T) -> Self {
		Self {
			data: RwLock::new(value),
			subscriptions: Mutex::new(Subscriptions::new()),
			version: AtomicUsize::new(0),
		}
	}
	pub fn try_borrow<'a>(&'a self) -> Result<RxGuard<'a, T>, TryLockError<RwLockReadGuard<T>>> {
		let guard = self.data.try_read()?;
		Ok(RxGuard { guard })
	}
	fn try_borrow_mut_base<'a, const SILENT: bool>(
		&'a self,
	) -> Result<RxGuardMutBase<'a, T, SILENT>, TryLockError<RwLockWriteGuard<T>>> {
		let guard = self.data.try_write()?;
		Ok(RxGuardMutBase { guard, rx: self })
	}
	pub fn try_borrow_mut<'a>(
		&'a self,
	) -> Result<RxGuardMut<'a, T>, TryLockError<RwLockWriteGuard<T>>> {
		self.try_borrow_mut_base()
	}
	pub fn try_borrow_mut_silent<'a>(
		&'a self,
	) -> Result<RxGuardMutSilent<'a, T>, TryLockError<RwLockWriteGuard<T>>> {
		self.try_borrow_mut_base()
	}
	fn get_version(&self) -> usize {
		self.version.load(std::sync::atomic::Ordering::SeqCst)
	}
	fn increment_version(&self) {
		self.version
			.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
	}
	fn with_subscriptions<U, F: FnOnce(&mut Subscriptions) -> U>(&self, func: F) -> U {
		let mut locked = self.subscriptions.lock().unwrap();
		func(&mut *locked)
	}
	pub fn into_inner(self) -> T {
		self.data.into_inner().unwrap()
	}
}
