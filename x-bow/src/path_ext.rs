use std::cell::Ref;

use crate::{borrow_mut_guard::BorrowMutGuard, until_change::UntilChange, Path, Trackable};

/// An extension trait with methods for [Path].
///
/// Most of the time, you'll be working with methods from this trait
/// (rather than the ones from [Path]).
///
/// This trait is implemented for every type that implements [Path].
pub trait PathExt: Path {
    /// Borrow the data at this path immutably.
    ///
    /// Returns None if the data identified by this path does not exist at
    /// the moment. For example...
    /// *   The path might point to data in an enum variant,
    ///     but the enum might be in a different variant.
    /// *   The path might point to data associated with a specific key in
    ///     a HashMap, but the HashMap might not have data at that key.
    ///
    /// If there is an existing mutable borrow anywhere in the
    /// store, this method will panic.
    ///
    /// See also: [borrow][crate::PathExtGuaranteed::borrow] - like this method,
    /// but for cases where we know None won't be returned.
    fn borrow_opt(&self) -> Option<Ref<'_, <Self as Path>::Out>> {
        self.path_borrow()
    }
    /// Borrow the data at this path mutably, notifying all the relevant
    /// change listeners when the returned borrow guard is dropped.
    ///
    /// Returns None if the data identified by this path does not exist at
    /// the moment. For example...
    /// *   The path might point to data in an enum variant,
    ///     but the enum might be in a different variant.
    /// *   The path might point to data associated with a specific key in
    ///     a HashMap, but the HashMap might not have data at that key.
    ///
    /// If there is an existing mutable or immutable borrow anywhere in the
    /// store, this method will panic.
    ///
    /// See also: [borrow_mut][crate::PathExtGuaranteed::borrow_mut] -
    /// like this method, but for cases where we know None won't be returned.
    fn borrow_opt_mut(&self) -> Option<BorrowMutGuard<'_, Self>> {
        self.path_borrow_mut()
            .map(|inner| BorrowMutGuard::new(inner, self.store_wakers(), self))
    }
    /// Get a [Stream][futures_core::Stream] that fires everytime a mutable
    /// borrow is taken of this or any encompassing piece of data.
    ///
    /// In other words, whenever someone call [borrow_opt_mut][Self::borrow_opt_mut]
    /// or [borrow_mut][crate::PathExtGuaranteed::borrow_mut] on this path
    /// (the same one you're calling `until_change` on) or any path that is a
    /// prefix of this one, the stream will fire.
    ///
    /// **The stream may fire spuriously**.
    /// Although the chance of this happening is extremely low.
    ///
    /// ```
    /// # use x_bow::{Trackable, Store, PathExt};
    ///
    /// #[derive(Default, Trackable)]
    /// struct MyStruct {
    ///     field_1: i32,
    ///     field_2: u64
    /// }
    /// let store = Store::new(MyStruct::default());
    ///
    /// // path to the `MyStruct` itself
    /// let path = store.build_path();
    ///
    /// let stream = path.until_change();
    ///
    /// path.borrow_opt_mut() // will fire the stream
    /// path.field_1().borrow_opt_mut() // will fire the stream
    /// path.field_2().borrow_opt_mut() // won't fire the stream
    /// ```
    fn until_change(&self) -> UntilChange<'_> {
        UntilChange::new(self.store_wakers(), self)
    }
    /// Get a [Stream][futures_core::Stream] that fires everytime a mutable
    /// borrow is taken of this or any encompassing piece of data.
    ///
    /// In other words, whenever someone call [borrow_opt_mut][Self::borrow_opt_mut]
    /// or [borrow_mut][crate::PathExtGuaranteed::borrow_mut] on this path
    /// (the same one you're calling `until_bubbling_change` on) or any path that this
    /// path is a prefix of, the stream will fire.
    ///
    /// **The stream may fire spuriously**.
    /// Although the chance of this happening is extremely low.
    ///
    /// ```
    /// # use x_bow::{Trackable, Store, PathExt};
    ///
    /// #[derive(Trackable)]
    /// struct MyStruct<T> {
    ///     field_1: T,
    ///     field_2: u64
    /// }
    /// let store = Store::new(MyStruct {
    ///     field_1: MyStruct {
    ///         field_1: String::new(),
    ///         field_2: 123
    ///     },
    ///     field_2: 456
    /// });
    ///
    /// // path to `field_1` in the root `MyStruct`
    /// let path = store.build_path().field_1();
    ///
    /// let stream = path.until_bubbling_change();
    ///
    /// path.field_1().borrow_opt_mut() // will fire the stream
    /// path.field_1().field_1().borrow_opt_mut() // will fire the stream
    /// path.field_1().field_2().borrow_opt_mut() // will fire the stream
    /// path.borrow_opt_mut() // won't fire the stream
    /// path.field_2().borrow_opt_mut() // won't fire the stream
    /// ```
    fn until_bubbling_change(&self) -> UntilChange<'_> {
        UntilChange::new_bubbling(self.store_wakers(), self)
    }

    /// Gives a [PathBuilder][crate::Trackable::PathBuilder] for creating a
    /// path that continues from this path.
    ///
    /// This is useful when you are handling the type that implements `Path`
    /// directly. Most of the time, though, you will already be working with
    /// `PathBuilder`s.
    fn build_path(self) -> <Self::Out as Trackable>::PathBuilder<Self>
    where
        Self: Sized,
        Self::Out: Trackable,
    {
        <Self::Out as Trackable>::new_path_builder(self)
    }
}

/// Extension trait is blanket-implemented.
impl<P: Path> PathExt for P {}
