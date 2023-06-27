/*!
## X-Bow: Fine-Grained Change Tracking for Rust Types

This crate provides a way to track fine-grained change in Rust structs and enums.

## Quick Example

```rust
# use x_bow::{Trackable, Tracked, TrackedGuaranteed, create_store};
# #[derive(Trackable)]
# struct MyStruct {
#   field_1: u32,
#   field_2: &'static str
# }
# let _ = async {
let data = MyStruct {
    field_1: 15,
    field_2: "asdf"
};
let store = create_store(data);

*store.field_1.borrow_mut() += 1;
store.field_2.until_change().await;
# };
```

## Workflow

1. You declare your structs and enums with `#[derive(Trackable)]`.
1. Construct your data and pass it into [create_store].
1. You can use `borrow` or `borrow_mut` to access parts of the data; for example,
you can borrow only a specific field in your struct.
1. You can subscribe to changes in certain parts of the data; for example,
you can ask to be notified when a specific field in your struct gets mutable borrowed.
This subscription is exposed through the async `until_change()` method.

## Usage

### The `Trackable` Derive Macro

Put `#[derive(Trackable)]` on your struct or enum to track its fields.

#### Deep and Shallow Tracking

By default, `#[derive(Trackable)]` does *shallow* tracking.
This means it tracks the fields of the struct, but only as opaque pieces of data.

*Deep* tracking, on the other hand, tracks each field in the struct as `Trackable` itself,
so you can access the fields inside each field, etc.

To choose what kind of tracking you want for each field put
* `#[track(deep)]` for deep tracking
* `#[track(shallow)]` or just `#[track]` for shallow tracking
* `#[track(skip)]` for no tracking

The attribute can also be put on the struct/enum declaration itself.
In that case it applies to every field by default and can be overridden for each.

```
# use x_bow::{Trackable, Tracked, create_store};
#[derive(Trackable)]
struct MyStruct {
    #[track(deep)]
    my_field: OtherStruct
}
#[derive(Trackable)]
struct OtherStruct {
    other_field: i32
}
let store = create_store(MyStruct {
    my_field: OtherStruct {
        other_field: 5
    }
});

# let _ = async {
// This works regardless of deep or shallow tracking on `MyStruct`
store.my_field.until_change().await;

// This requires deep tracking on `MyStruct`
store.my_field.other_field.until_change().await;
# };
```

### Projecting into Enums

You can ask to borrow certain variants of an enum. For example,
```rust
# use x_bow::{Tracked, TrackedGuaranteed, create_store};
let data: Option<i32> = Some(123);
let store = create_store(data);
if let Some(mut x) = store.Some_0.borrow_opt_mut() {
    *x += 1;
};
assert_eq!(*store.borrow(), Some(124));
```
You have to use `borrow_opt` and `borrow_opt_mut`, which will only return
`Some` if the enum is in the variant you asked for.

### Listening for Changes

#### *Inside*, *Here*, and *Outside* Changes

Think of your data as a tree; the full data is the root node;
each field in that struct is a child node, on and on.

```text
                                    ┌─────────────┐
                                    │root: Struct1│
                                    └──────┬──────┘
                                           │
                           ┌───────────────┴──────────────┐
                           │                              │
                   ┌───────▼─────────────┐        ┌───────▼────────────┐
                   │field_1: Vec<Struct2>│        │field_2: Option<i32>│
                   └───────┬─────────────┘        └───────┬────────────┘
                           │                              │
  ┌─────────────┬──────────┴──┬─────────────┐             └───────┐
  │             │             │             │                     │
┌─▼────────┐  ┌─▼────────┐  ┌─▼────────┐  ┌─▼────────┐     ┌──────▼────┐
│0: Struct2│  │1: Struct2│  │2: Struct2│  │3: Struct2│     │Some_0: i32│
└──────────┘  └──────────┘  └──────────┘  └──────────┘     └───────────┘
```

Changes are fired when a mutable borrow is taken at a node in the tree.
* *Here* changes are fired for the node where the mutable borrow was taken.
* *Outside* changes are fired for all descendants of the mutably borrowed node.
* *Inside* changes are fired for all ancestors of the mutably borrowed node.

For example, if `field_1: Vec<Struct2>` in the diagram above was borrowed mutably
(`store.root.field_1.borrow_mut();`)...
* *Here* change would fire for `field_1: Vec<Struct2>`.
* *Inside* change would fire for the ancestors - the `root: Struct1` node.
* *Outside* change would fire for the descendants
(`0: Struct2`, `1: Struct2`, `2: Struct2`, `3: Struct2`).

### Reading and Modifying Data

As you've already seen in the past example codes, data can be read and modified
through the borrow methods

When we can guarantee that the piece of data we want to read/modify exists,
we use the direct borrow methods
* [borrow][TrackedGuaranteed::borrow]
* [borrow_mut][TrackedGuaranteed::borrow_mut]
These methods require that the data is
* not inside an enum variant (the enum could be in a different variant!)
* not inside a Vec (the vector might not have the index we ask for)
* not inside a HashMap (the data at the key we want might not be in the map)

When the conditions are not satisfied, we need to use the optional methods
* [borrow_opt][Tracked::borrow_opt]
* [borrow_opt_mut][Tracked::borrow_opt_mut]

If you want to control the change signals fired from your `borrow_mut` calls,
use
* [borrow_mut_custom][Tracked::borrow_mut_custom]
* [borrow_opt_mut_custom][Tracked::borrow_mut_custom]


*/

mod impls;
mod is_guaranteed;
mod listeners;
mod mapper;
mod node_down;
mod node_up;
mod track_api;
mod track_root;
mod trackable;
mod until_change;

pub use track_api::{Store, Tracked, TrackedGuaranteed};
pub use track_root::create_store;
pub use trackable::Trackable;
pub use until_change::UntilChange;
pub use x_bow_macros::Trackable;

#[doc(hidden)]
pub mod __private_macro_only {
    pub use super::impls::leaf::{NodeDownLeaf, TrackableLeaf};
    pub use super::is_guaranteed::IsGuaranteed;
    pub use super::mapper::Mapper;
    pub use super::node_down::NodeDownTrait;
    pub use super::node_up::{NodeUp, NodeUpTrait};
    pub use super::trackable::Trackable;
    pub use bool;
}
