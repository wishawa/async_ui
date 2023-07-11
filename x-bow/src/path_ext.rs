use std::cell::Ref;

use crate::{borrow_mut_guard::BorrowMutGuard, until_change::UntilChange, Path, Trackable};

pub trait PathExt: Path {
    /// Borrow the data at the given path immutably.
    ///
    /// If there is an existing mutable borrow anywhere in the
    /// store, this method will panic.
    fn borrow_opt<'d>(&'d self) -> Option<Ref<'d, <Self as Path>::Out>>
    where
        Self: 'd,
    {
        self.path_borrow()
    }
    /// Borrow the data at the given path mutably, notifying all the relevant
    /// change listeners when the borrow guard is dropped.
    ///
    /// If there is an existing mutable or immutable borrow anywhere in the
    /// store, this method will panic.
    fn borrow_opt_mut<'d>(&'d self) -> Option<BorrowMutGuard<'d, Self>> {
        self.path_borrow_mut()
            .map(|inner| BorrowMutGuard::new(inner, self.store_wakers(), self))
    }
    fn until_change<'d>(&'d self) -> UntilChange<'d> {
        UntilChange::new(self.store_wakers(), self)
    }
    fn until_bubbling_change<'d>(&'d self) -> UntilChange<'d> {
        UntilChange::new_bubbling(self.store_wakers(), self)
    }
    fn build_path(self) -> <Self::Out as Trackable>::PathBuilder<Self>
    where
        Self: Sized,
        Self::Out: Trackable,
    {
        <Self::Out as Trackable>::new_path_builder(self)
    }
}

impl<P: Path> PathExt for P {}
