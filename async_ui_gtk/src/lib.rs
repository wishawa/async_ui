#![deny(unsafe_op_in_unsafe_fn)]
mod backend;
mod element;
mod executor;
mod render;
mod vnode;
mod wrappers;

pub use element::Element;
pub use render::{mount_and_present, render};
pub use wrappers::*;
pub mod manual_apis {
    pub use super::backend::GtkBackend;
    pub use super::executor::GtkSpawner;
    pub use super::render::{put_node, render_in_node};
    pub use super::vnode::ContainerHandler;
    pub type NodeGuard = async_ui_core::local::control::node_guard::NodeGuard<GtkBackend>;
    pub type RenderFuture<'e> = async_ui_core::local::render::RenderFuture<'e, GtkBackend>;
}
