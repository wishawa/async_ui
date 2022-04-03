use std::rc::Rc;

use async_ui_core::local::{backend::Backend, control::Control};

use crate::{
    executor::GtkSpawner,
    vnode::{NullVNode, VNode, VNodeEnum},
};

scoped_tls::scoped_thread_local!(
    static CONTROL: Control<GtkBackend>
);
thread_local! {
    static DUMMY_CONTROL: Control<GtkBackend> = Control::new_with_vnode(VNode(Rc::new(VNodeEnum::from(NullVNode))));
}

pub struct GtkBackend;
impl Backend for GtkBackend {
    type VNode = VNode;

    type Spawner = GtkSpawner;

    fn get_tls() -> &'static scoped_tls::ScopedKey<Control<Self>> {
        &CONTROL
    }

    fn get_dummy_control() -> Control<Self> {
        DUMMY_CONTROL.with(Clone::clone)
    }
}
