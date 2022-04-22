use std::pin::Pin;

use slab::Slab;

use crate::{
    backend::Backend, control::Control, drop_check::check_drop_scope, element::Element,
    render::Render,
};

pin_project_lite::pin_project! {
    pub struct ManyRender<'e, B: Backend> {
        renders: Slab<Element<'e, B>>
    }
}
#[derive(Clone, Copy, Default)]
pub struct ManyRenderKey(usize);

impl<'e, B: Backend> ManyRender<'e, B> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            renders: Slab::with_capacity(capacity),
        }
    }
    pub fn add_render(
        self: Pin<&mut Self>,
        render: Render<'e, B>,
        control: Control<B>,
    ) -> ManyRenderKey {
        check_drop_scope(&*self as *const _ as *const ());
        let this = self.project();
        let mut elem: Element<'e, B> = render.into();
        unsafe { elem.mount(control) };
        let key = this.renders.insert(elem);
        ManyRenderKey(key)
    }
    pub fn remove_render(self: Pin<&mut Self>, key: ManyRenderKey) {
        let this = self.project();
        let elem = this.renders.remove(key.0);
        std::mem::drop(elem);
    }
}
