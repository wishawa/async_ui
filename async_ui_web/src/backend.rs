use std::rc::Rc;

use async_ui_core::{
	backend::Backend,
	control::{vnode::null::NullVNode, Control},
};
use web_sys::Node;

pub struct WebBackend;

scoped_tls::scoped_thread_local!(
	static CONTROL: Control<WebBackend>
);
thread_local! {
	static DUMMY_CONTROL: Control<WebBackend> = Control::new_with_vnode(Rc::new(NullVNode));
}

impl Backend for WebBackend {
	type NodeType = Node;

	fn get_tls() -> &'static scoped_tls::ScopedKey<Control<Self>> {
		&CONTROL
	}

	fn get_dummy_control() -> Control<Self> {
		DUMMY_CONTROL.with(Clone::clone)
	}
}
