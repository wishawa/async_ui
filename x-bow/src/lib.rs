mod edge;
mod impls;
mod listeners;
mod mapper;
mod notify_guard;
mod observable;
mod optional;
mod store;
mod trackable;
mod tracked;
pub use x_bow_macros::Track;

#[doc(hidden)]
pub mod __private_macro_only {
    pub use super::edge::{Edge, TrackedEdge};
    pub use super::impls::XBowLeaf;
    pub use super::mapper::{ClosureMapper, Mapper};
    pub use super::optional::{IsOptional, OptionalNo, OptionalYes};
    pub use super::trackable::Trackable;
    pub use super::tracked::{Tracked, TrackedNode, TrackedNodeAlias};
}
pub use store::{create_store, Store};

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
