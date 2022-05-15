use std::rc::Rc;

use async_ui_core::{
	control::Control,
	render::{
		put_node as base_put_node, set_render_control as base_set_render_control, spawn_root,
		NodeGuard,
	},
	runtime::drive_runtime,
};
use web_sys::Node;

use crate::{backend::WebBackend, executor::WebSpawner, vnode::NodeVNode};

pub type Render<'e> = async_ui_core::render::Render<'e, WebBackend>;

pub fn put_node(node: Node) -> NodeGuard<WebBackend> {
	base_put_node::<WebBackend>(node)
}
pub fn control_from_node(node: Node) -> Control<WebBackend> {
	Control::new_with_vnode(Rc::new(NodeVNode::new(node)))
}
pub fn set_render_control<'e>(render: &mut Render<'e>, control: Control<WebBackend>) {
	base_set_render_control(render, control);
}

pub fn mount_at(children: impl Into<Render<'static>>, node: Node) {
	let control = Control::new_with_vnode(Rc::new(NodeVNode::new(node)));
	let mut children = children.into();
	set_render_control(&mut children, control);
	let task = spawn_root(children);
	task.detach();
	WebSpawner::set_future(drive_runtime());
	WebSpawner::schedule_now();
}

pub fn mount(children: impl Into<Render<'static>>) {
	let node = web_sys::window()
		.unwrap()
		.document()
		.unwrap()
		.body()
		.unwrap();
	mount_at(children, node.into());
}
