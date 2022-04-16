#![deny(unsafe_op_in_unsafe_fn)]
mod backend;
mod executor;
mod render;
mod vnode;
mod wrappers;

pub use render::{mount, Render};
pub use wrappers::*;
pub mod manual_apis {
    pub use super::backend::WebBackend;
    pub use super::executor::WebSpawner;
    pub use super::render::{control_from_node, put_node, set_render_control};
    pub type NodeGuard = async_ui_core::control::node_guard::NodeGuard<WebBackend>;
}
