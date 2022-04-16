mod shared;
use std::cell::{BorrowError, BorrowMutError, Cell, Ref, RefCell, RefMut};

use self::shared::{RxGuard, RxGuardMut, RxGuardMutBase, RxGuardMutSilent};

use super::subscriptions::Subscriptions;

pub struct Rx<T> {
    data: RefCell<T>,
    subscriptions: RefCell<Subscriptions>,
    version: Cell<usize>,
}

type RefRead<'a, T> = Ref<'a, T>;
type RefWrite<'a, T> = RefMut<'a, T>;

impl<T> Rx<T> {
    pub fn new(value: T) -> Self {
        Self {
            data: RefCell::new(value),
            subscriptions: RefCell::new(Subscriptions::new()),
            version: Cell::new(0),
        }
    }
    pub fn try_borrow<'a>(&'a self) -> Result<RxGuard<'a, T>, BorrowError> {
        let guard = self.data.try_borrow()?;
        Ok(RxGuard { guard })
    }
    fn try_borrow_mut_base<'a, const SILENT: bool>(
        &'a self,
    ) -> Result<RxGuardMutBase<'a, T, SILENT>, BorrowMutError> {
        let guard = self.data.try_borrow_mut()?;
        Ok(RxGuardMutBase { guard, rx: self })
    }
    pub fn try_borrow_mut<'a>(&'a self) -> Result<RxGuardMut<'a, T>, BorrowMutError> {
        self.try_borrow_mut_base()
    }
    pub fn try_borrow_mut_silent<'a>(&'a self) -> Result<RxGuardMutSilent<'a, T>, BorrowMutError> {
        self.try_borrow_mut_base()
    }

    fn get_version(&self) -> usize {
        self.version.get()
    }
    fn increment_version(&self) {
        self.version.set(self.version.get() + 1);
    }
    fn with_subscriptions<U, F: FnOnce(&mut Subscriptions) -> U>(&self, func: F) -> U {
        let mut locked = self.subscriptions.borrow_mut();
        func(&mut *locked)
    }
}
