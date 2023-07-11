use std::cell::Ref;

use crate::{borrow_mut_guard::BorrowMutGuard, path_ext::PathExt, Path};

/// Implemented for [Path] objects that will always be able to obtain their
/// data piece.
///
/// Such paths are those that
/// *   don't go through any `enum` (if the enum is in a different variant,
///     then the data cannot be obtained)
/// *   don't go through `Vec` or `HashMap` (the requested item might not
///     be in the collection)
///
/// The two methods in this trait are similar to [borrow_opt][PathExt::borrow_opt]
/// and [borrow_opt_mut][PathExt::borrow_opt_mut], but they don't return [Option]
/// because we know the data is always there.
pub trait PathExtGuaranteed: PathExt {
    /// Borrow the data at this path immutably.
    ///
    /// See [borrow_opt][PathExt::borrow_opt] for more details.
    fn borrow<'d>(&'d self) -> Ref<'d, <Self as Path>::Out>
    where
        Self: 'd,
    {
        self.borrow_opt().unwrap()
    }
    /// Borrow the data at this path mutably, notifying all the relevant
    /// change listeners when the returned borrow guard is dropped.
    ///
    /// See [borrow_opt_mut][PathExt::borrow_opt_mut] for more details.
    fn borrow_mut<'d>(&'d self) -> BorrowMutGuard<'d, Self>
    where
        Self: 'd,
    {
        self.borrow_opt_mut().unwrap()
    }
}
