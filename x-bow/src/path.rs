use std::cell::{Ref, RefCell, RefMut};

use crate::{hash_visitor::HashVisitor, wakers::StoreWakers};

/// A Path object identifies a piece of the data store.
/// It can convert a borrow of the store's data to a borrow of the piece it identifies.
///
/// Path objects are usually built compositionally. Take for example
/// ```rust
/// struct Root {
///     field_1: Type1
/// }
/// enum Type1 {
///     VariantA(i32, u64),
///     VariantB{
///         strings: Vec<String>
///     }
/// }
/// ```
/// a path from the root to a `String` might look like this
/// ```text
/// VecToItem(
///     Type1_VariantB_strings(
///         Root_field1()
///     ),
///     <item index in the Vec>
/// )
/// ```
///
pub trait Path {
    type Out: ?Sized;

    /// Borrow the data at the given path immutably.
    ///
    /// If there is an existing mutable borrow anywhere in the store,
    /// this method will panic.
    fn path_borrow<'d>(&'d self) -> Option<Ref<'d, Self::Out>>
    where
        Self: 'd;

    /// Borrow the data at the given path mutably.
    ///
    /// If there is an existing mutable or immutable borrow anywhere in the store,
    /// this method will panic.
    fn path_borrow_mut<'d>(&'d self) -> Option<RefMut<'d, Self::Out>>
    where
        Self: 'd;

    /// Call the given visitor function on the hash of this path node and every
    /// ancestor node in the path.
    fn visit_hashes(&self, visitor: &mut HashVisitor);

    /// Used internally for subscription and notification system.
    fn store_wakers(&self) -> &RefCell<StoreWakers>;
}
