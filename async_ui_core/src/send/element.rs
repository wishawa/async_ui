use std::{future::Future, marker::PhantomData, pin::Pin};

use super::{
    backend::{Backend, Spawner},
    control::Control,
    drop_check::PropagateDropScope,
    MaybeSend,
};

pin_project_lite::pin_project! {
    struct ElementInner<'e, B: Backend, F: Future<Output = ()>> {
        control: Control<B>,
        _lifetime: PhantomData<&'e ()>,
        #[pin]
        future: F
    }
}

trait ElementTrait<B: Backend>: Future<Output = ()> + MaybeSend {
    fn set_control(self: Pin<&mut Self>, control: Control<B>);
}

pub struct Element<'e, B: Backend>(Pin<Box<dyn ElementTrait<B> + 'e>>);

impl<'e, B: Backend> Element<'e, B> {
    pub(crate) fn set_control(&mut self, control: Control<B>) {
        self.0.as_mut().set_control(control);
    }
    pub(crate) unsafe fn spawn(self) -> <B::Spawner as Spawner>::Task {
        let fut = self.0;
        let erased = unsafe {
            std::mem::transmute::<
                Pin<Box<dyn ElementTrait<B> + 'e>>,
                Pin<Box<dyn ElementTrait<B> + 'static>>,
            >(fut)
        };
        let fut = PropagateDropScope::new(erased);
        <B::Spawner as Spawner>::spawn(fut)
    }
}

impl<'e, B: Backend + 'e, F: Future<Output = ()>> Future for ElementInner<'e, B, F> {
    type Output = ();
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        B::get_tls().set(this.control, || this.future.poll(cx))
    }
}
impl<'e, B: Backend + 'e, F: Future<Output = ()> + MaybeSend> ElementTrait<B>
    for ElementInner<'e, B, F>
{
    fn set_control(self: Pin<&mut Self>, control: Control<B>) {
        let this = self.project();
        *this.control = control;
    }
}

impl<'e, F: Future<Output = ()> + MaybeSend + 'e, B: Backend> From<F> for Element<'e, B> {
    fn from(future: F) -> Self {
        Element(Box::pin(ElementInner {
            control: B::get_dummy_control(),
            _lifetime: PhantomData,
            future,
        }))
    }
}
