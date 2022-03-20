use crate::control::element_control::{ElementControl, ELEMENT_CONTROL};
use std::{future::Future, pin::Pin};

pub struct Element<'e>(Box<dyn ElementTrait<'e>>);
pin_project_lite::pin_project! {
struct ElementInner<F>
where F: Future<Output = ()>
{
    control: ElementControl,
    #[pin]
    future: F
}
}
impl<'e> Element<'e> {
    pub(crate) fn set_control(&mut self, control: ElementControl) {
        self.0.set_control(control)
    }
    pub(crate) fn to_boxed_future(self: Self) -> Pin<Box<dyn Future<Output = ()> + 'e>> {
        self.0.to_boxed_future()
    }
}

trait ElementTrait<'e>: 'e {
    fn to_boxed_future(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + 'e>>;
    fn set_control(&mut self, control: ElementControl);
}
impl<'e, F: Future<Output = ()> + 'e> ElementTrait<'e> for ElementInner<F> {
    fn to_boxed_future(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + 'e>> {
        let boxed = self as Box<dyn Future<Output = ()> + 'e>;
        boxed.into()
    }
    fn set_control(&mut self, control: ElementControl) {
        self.control = control;
    }
}
impl<'e, F: Future<Output = ()> + 'e> Future for ElementInner<F> {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();
        ELEMENT_CONTROL.set(this.control, || this.future.as_mut().poll(cx))
    }
}

impl<'e, F: Future<Output = ()> + 'e> From<F> for Element<'e> {
    fn from(future: F) -> Self {
        let control = ElementControl::get_dummy();
        let inner = ElementInner { control, future };
        Self(Box::new(inner) as _)
    }
}
