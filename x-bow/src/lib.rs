//! # X-Bow: precise state management
//!
//! X-Bow is a state management library aimed for use in UI programming.
//! It let you...
//! * keep your data in a centralized store.
//! * build "paths" that point to parts of the store.
//! * borrow and mutate data at those paths.
//! * subscribe to mutations at those paths through async API.
//!
//! ## Quick Example
//!
//! ```
//! # use x_bow::{Trackable, Store, PathExt, PathExtGuaranteed};
//! # #[derive(Default, Trackable)]
//! # struct AnotherStruct {
//! #     asdf: String
//! # }
//! // Derive `Trackable` to allow parts of the struct to be tracked.
//! #[derive(Default, Trackable)]
//! #[track(deep)] // `deep` option is useful if the fields themselves are structs
//! struct MyStruct {
//!     field_1: i32,
//!     field_2: u64,
//!     child_struct: AnotherStruct
//! }
//!
//! // Create a centralized store with the data.
//! let store = Store::new(MyStruct::default());
//!
//! // Build a path to the `i32` at `field_1` in the `MyStruct`.
//! let path = store.build_path().field_1();
//!
//! // This implements the `Stream` trait. You can do `stream.next().await`, etc.
//! let stream = path.until_change();
//!
//! // Mutably borrow the `i32` of the path, and increment it.
//! *path.borrow_mut() += 1;
//! ```
//!
//!
//! ## Concepts
//!
//! ### Store
//! The store is where the application state lives. It is a big RefCell.
//!
//! ### Paths
//! A path identifies a piece of data in your store. It implements [PathExt],
//! which contains most of the methods you will interact with.
//!
//! Paths are usually wrapped in `PathBuilder` objects. These objects each
//! dereference to the path object it wraps.
//!
//! ### Path Builders
//! A path builder wraps a path object and derefs to the path object.
//!
//! It also provides methods that to "continue" the path; if the path points to
//! `T`, the PathBuilder will let you convert it to a path that points to some
//! part inside `T`.
//!
//! For example: the path builder that wraps a path to `Vec<T>` has an
//! `index(idx: usize)` method that returns a path to `T`.
//!
//! ### Trackable Types
//! Types that implements [Trackable] have their corresponding `PathBuilder` type.
//! To be `Trackable`, types should have `#[derive(Trackable)]` on them.
//!
//! ## Design
//!
//! ### Borrowing and Paths
//! The design of this library is a hybrid between simple [RefCell][std::cell::RefCell]
//! and the "lens" concept prevalent in the functional programming world.
//!
//! The centralized data store of X-Bow is a RefCell.
//! The library provides RefCell-like APIs like
//! [borrow][PathExtGuaranteed::borrow] and [borrow_mut][PathExtGuaranteed::borrow_mut].
//! Mutation is well supported and immutable data structures are not needed.
//!
//! The library uses *Path*s to identify parts of the data in the store.
//! Paths are made from composing segments. Each segment is like a "lens",
//! knowing how to project into some substructure of a type.
//! Composed together, these segments become a path that knows how to project
//! from the root data in the store to the part that it identifies.
//!
//! The difference between Paths and Lens/Optics is just that our paths work
//! mutably, while Lens/Optics are often associated with immutable/functional design.
//!
//! ### Notification and Subscription
//! Another important aspect of the library design is the notification/subscription
//! functionality provided through [until_change][PathExt::until_change]
//! and [until_bubbling_change][PathExt::until_bubbling_change].
//!
//! Change listening is done based on Paths' hashes. We have a map associating
//! each hash to a version number and a list of listening [Waker][std::task::Waker]s.
//! The [until_change][PathExt::until_change] method registers wakers at the
//! hash of its target path and all prefix paths (when some piece of data
//! encompassing the target data is changed, we assume the target data is changed too).
//!
//! #### Hash Collision
//! If two paths end up with the same hash, wake notification to one would wake
//! listeners to the other too. Thus, the `until_change` stream may fire spuriously.
//!
//! That said, the probability of `u64` hash collision is extremely low.
//! With 10,000 distinct paths in a store, collision probability can
//! [be calculated](https://en.wikipedia.org/wiki/Birthday_problem#Probability_of_a_shared_birthday_(collision))
//! to be less than 1E-11 (0.0000000000001%).
//!

mod borrow_mut_guard;
mod guarantee;
mod hash;
mod hash_visitor;
mod impls;
mod path;
mod path_ext;
mod path_impl;
mod store;
mod trackable;
mod tracked;
mod until_change;
mod wakers;

pub use guarantee::PathExtGuaranteed;
pub use path::Path;
pub use path_ext::PathExt;
pub use path_impl::ReferencePath;
pub use store::{Store, StoreRoot};
pub use trackable::Trackable;
pub use tracked::{Tracked, TrackedGuaranteed};

/// Macro to allows building paths to fields inside a struct/enum.
///
/// ```
/// # use x_bow::{Trackable, Store, PathExt, PathExtGuaranteed};
/// // Derive `Trackable` to allow parts of the struct to be tracked.
/// #[derive(Default, Trackable)]
/// struct MyStruct {
///     field_1: i32,
///     field_2: u64
/// }
///
/// // Create a centralized store with the data.
/// let store = Store::new(MyStruct::default());
///
/// // Build a path to the `i32` at `field_1` in the `MyStruct`.
/// let path = store.build_path().field_1();
/// ```
///
/// ### Shallow and Deep Tracking
///
/// By default, the `Trackable` derive macro makes tracking "shallow".
/// This means you can build path into each child field of a struct/enum,
/// but if the type in a child field is itself a struct/enum, you won't be able
/// to continue your path into a grandchild field.
/// ```compile_fail
/// # use x_bow::{Trackable, Store, PathExt, PathExtGuaranteed};
/// #[derive(Default, Trackable)]
/// struct MyStruct {
///     field_1: ChildStruct,
///     field_2: u64
/// }
/// #[derive(Default)]
/// struct ChildStruct {
///     asdf: String
/// }
///
/// let store = Store::new(MyStruct::default());
///
/// store.build_path().field_1(); // OK
/// store.build_path().field_1().asdf(); // cannot do this!!
/// ```
///
/// To allow building paths deep into descendants, enable deep tracking
/// ```
/// # use x_bow::{Trackable, Store, PathExt, PathExtGuaranteed};
/// #[derive(Default, Trackable)]
/// struct MyStruct {
///     #[track(deep)] // 👈 enable deep tracking
///     field_1: ChildStruct,
///     field_2: u64
/// }
/// #[derive(Default, Trackable /* 👈 the subject of deep tracking must be Trackable */)]
/// struct ChildStruct {
///     asdf: String
/// }
///
/// let store = Store::new(MyStruct::default());
///
/// store.build_path().field_1(); // OK
/// store.build_path().field_1().asdf(); // OK
/// ```
///
/// Often, you will end up with `#[track(deep)]` on most (if not all) your
/// fields. In this case, you can apply the attribute on the struct/enum itself.
/// The attribute `#[track(shallow)]` can then be applied on individual fields
/// to opt out of deep tracking.
/// ```
/// # use x_bow::{Trackable, Store, PathExt, PathExtGuaranteed};
/// #[derive(Trackable)]
/// #[track(deep)] // 👈 enable deep tracking
/// struct MyStruct {
///     field_1: ChildStruct,
///     #[track(shallow)] // 👈 don't track this field deeply
///     field_2: u64
/// }
/// # #[derive(Trackable)]
/// # struct ChildStruct {
/// #     asdf: String
/// # }
/// ```
///
/// ### Tracking Enums
///
/// As implied along this doc, the `Trackable` derive macro works on enums too.
/// ```
/// # use x_bow::{Trackable, Store, PathExt, PathExtGuaranteed};
/// #[derive(Trackable)]
/// enum MyEnum {
///     Variant1 (String),
///     Variant2 {
///         field_a: i32,
///         field_b: u64
///     }
/// }
/// let store = Store::new(MyEnum::Variant1 ("Hello".into()));
///
/// let path = store.build_path().Variant1_0(); // <- the `String` in Variant1
/// let path = store.build_path().Variant2_field_a(); // <- the `i32` in Variant2
/// ```
pub use x_bow_macros::Trackable;

#[doc(hidden)]
pub mod __private_macro_only {
    pub use super::guarantee::PathExtGuaranteed;
    pub use super::hash_visitor::HashVisitor;
    pub use super::impls::leaf::TrackableLeaf;
    pub use super::path::Path;
    pub use super::trackable::IntoInnerPath;
    pub use super::trackable::Trackable;
    pub use super::wakers::StoreWakers;
    pub use x_bow_macros::IntoInnerPath;
}
