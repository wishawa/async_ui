//! Different components for rendering a collection of things.
//!
//! There are many ways to render a list of items with Async UI Web.
//!
//! ### Basic `join`/`race`
//! The [join][crate::join] and [race][crate::race] functions support array or
//! [Vec] of futures as input. They display the futures side by side, in order.
//!
//! ```
//! # use async_ui_web::{join, prelude_traits::*};
//! # let _ = async {
//!		join(
//! 		(1..20)
//! 			.map(|num| num.to_string().render())
//! 			.collect::<Vec<_>>()
//! 	).await;
//! # };
//! ```
//!
//! This is the most basic way to render a list of things. You don't need any
//! list component to do this.
//!
//! This approach is not very flexible: there is no way to insert into or remove
//! from the vec once you've passed it to `join`.
//!
//! ### DiffedList
//! [DiffedList] is more flexible. You provide it with a Vec of "keys", and
//! a closure to convert each key into a future. You can update the keys Vec
//! and the list will automatically make sure the right futures are rendered.
//!
//! DiffedList is not great for performance when the list is long.
//! In those case, prefer ModeledList...
//!
//! ### ModeledList
//! [ModeledList] is very similar to [DiffedList], but significantly more
//! performant. The tradeoff is you no longer work with a plain Vec of keys,
//! but rather a [ListModel]. ListModel is more restrictive than Vec on what
//! modification you can make, but common operations (insert/remove/swap/...)
//! are all available.
//!
//! **If you don't know what list component to use, use ModeledList**.
//!
//! ### DynamicList
//! [DynamicList] is the base component on which every other list builds on.
//! It provides fine control, but is harder to use. It's API is very imperative;
//! with DiffedList and ModeledList you update the "keys" and the UI
//! automatically gets updated, with DynamicList you have to deal with
//! inserting/moving/removing futures manually.
//!
//! ### VirtualizedList
//! [VirtualizedList] is very different from all the other lists. It is made
//! for very large collections. It only renders the items that are visible in
//! the viewport to save resource.
//!
//! The implementation still needs some work. Right now, the list only supports
//! fixed-height items.

mod diffed_list;
mod dynamic_list;
mod modeled_list;
mod virtualized_list;

pub use diffed_list::DiffedList;
pub use dynamic_list::DynamicList;
pub use modeled_list::{ListModel, ModeledList};
pub use virtualized_list::VirtualizedList;
