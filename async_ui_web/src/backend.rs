use std::rc::Rc;

use crate::{
    executor::WebSpawner,
    vnode::{NullVNode, VNodeEnum},
};
use async_ui_core::{backend::Backend, control::Control};

use super::vnode::VNode;

pub struct WebBackend;

scoped_tls::scoped_thread_local!(
    static CONTROL: Control<WebBackend>
);
thread_local! {
    static DUMMY_CONTROL: Control<WebBackend> = Control::new_with_vnode(VNode(Rc::new(VNodeEnum::from(NullVNode))));
}

impl Backend for WebBackend {
    type VNode = VNode;

    type Spawner = WebSpawner;

    fn get_tls() -> &'static scoped_tls::ScopedKey<async_ui_core::control::Control<Self>> {
        &CONTROL
    }

    fn get_dummy_control() -> Control<Self> {
        DUMMY_CONTROL.with(Clone::clone)
    }
}
