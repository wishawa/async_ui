#![deny(unsafe_op_in_unsafe_fn)]
mod backend;
mod render;
mod vnode;
mod wrappers;

pub use render::{mount_and_present, Render};
pub use wrappers::*;
pub mod manual_apis {
    pub use super::backend::GtkBackend;
    pub use super::render::{control_from_node, put_node, set_render_control};
    pub use super::vnode::ContainerHandler;
    pub type NodeGuard = async_ui_core::control::node_guard::NodeGuard<GtkBackend>;
}
