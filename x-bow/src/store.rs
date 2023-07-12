use std::cell::{Ref, RefCell, RefMut};

use crate::{guarantee::PathExtGuaranteed, path::Path, trackable::Trackable, wakers::StoreWakers};

/// A store is where your "state" data lives. It is in essence a big [RefCell].
/// There are also supporting mechanisms to enable subscriptions and
/// mutation notifications.
pub struct Store<S> {
    data: RefCell<S>,
    wakers: RefCell<StoreWakers>,
}

impl<S> Store<S> {
    /// Create a new store with the given data.
    /// This puts the data in a [RefCell] and set up all the change listening
    /// mechanisms.
    pub fn new(data: S) -> Self {
        Self {
            data: RefCell::new(data),
            wakers: RefCell::new(StoreWakers::new()),
        }
    }
}
impl<S: Trackable> Store<S> {
    /// Use this method to create paths to different pieces of your state.
    ///
    /// ```
    /// # use x_bow::{Trackable, Store, PathExt};
    ///
    /// #[derive(Trackable)]
    /// #[track(deep)]
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
    /// // path to the root `MyStruct` itself
    /// let path = store.build_path();
    ///
    /// // path to `field_1` in the root `MyStruct`
    /// let path = store.build_path().field_1();
    ///
    /// // path to `field_2` in the root `MyStruct`
    /// let path = store.build_path().field_2();
    ///
    /// // path root -> field_1 -> field_1
    /// let path = store.build_path().field_1().field_1();
    ///
    /// // path root -> field_1 -> field_2
    /// let path = store.build_path().field_1().field_2();
    /// ```
    pub fn build_path(&self) -> StoreRoot<'_, S> {
        S::new_path_builder(RootPath { store: self })
    }
}

/// The PathBuilder pointing to the root data type in the store itself.
///
/// This is obtained by [Store::build_path].
pub type StoreRoot<'s, S> = <S as Trackable>::PathBuilder<RootPath<'s, S>>;

/// The [Path] object to the root of the store.
pub struct RootPath<'s, S> {
    store: &'s Store<S>,
}

impl<'s, S> Clone for RootPath<'s, S> {
    fn clone(&self) -> Self {
        Self { store: self.store }
    }
}
impl<'s, S> Copy for RootPath<'s, S> {}

impl<'s, S> Path for RootPath<'s, S> {
    type Out = S;

    fn path_borrow(&self) -> Option<Ref<'_, Self::Out>> {
        Some(self.store.data.borrow())
    }

    fn path_borrow_mut(&self) -> Option<RefMut<'_, Self::Out>> {
        Some(self.store.data.borrow_mut())
    }

    fn visit_hashes(&self, visitor: &mut crate::hash_visitor::HashVisitor) {
        visitor.finish_one();
    }

    fn store_wakers(&self) -> &RefCell<StoreWakers> {
        &self.store.wakers
    }
}
impl<'s, S> PathExtGuaranteed for RootPath<'s, S> {}
