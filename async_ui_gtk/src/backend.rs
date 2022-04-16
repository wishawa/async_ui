use std::rc::Rc;

use async_ui_core::{
    backend::Backend,
    control::{vnode::null::NullVNode, Control},
};
use gtk::Widget;

scoped_tls::scoped_thread_local!(
    static CONTROL: Control<GtkBackend>
);
thread_local! {
    static DUMMY_CONTROL: Control<GtkBackend> = Control::new_with_vnode(Rc::new(NullVNode));
}

pub struct GtkBackend;
impl Backend for GtkBackend {
    type NodeType = Widget;

    fn get_tls() -> &'static scoped_tls::ScopedKey<Control<Self>> {
        &CONTROL
    }

    fn get_dummy_control() -> Control<Self> {
        DUMMY_CONTROL.with(Clone::clone)
    }
}
