use std::rc::Rc;

use futures_lite::FutureExt;
use observables::{cell::ReactiveCell, ObservableAs, ObservableAsExt};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Event, HtmlInputElement};

use crate::{get_context, window::DOCUMENT, with_context, Fragment};

use super::ElementFuture;

#[derive(Default)]
pub struct RadioProps<E: Clone + PartialEq + 'static> {
    pub value: E,
}

#[derive(Default)]
pub struct RadioGroupProps<'c, E: Clone + PartialEq + 'static> {
    pub children: Fragment<'c>,
    pub value: Option<&'c dyn ObservableAs<E>>,
    pub on_change: Option<&'c mut dyn FnMut(E)>,
}

struct RadioGroup<E: 'static> {
    name: String,
    value: ReactiveCell<E>,
}

pub async fn radio_group<'c, E: Clone + PartialEq + 'static>(
    RadioGroupProps {
        children,
        value,
        mut on_change,
    }: RadioGroupProps<'c, E>,
) {
    let ptr = &children as *const Fragment<'_>;
    let name = format!("radio-{ptr:x?}");
    let value = if let Some(v) = value { v } else { return };
    let value = ReactiveCell::new(value.borrow_observable_as().clone());
    let group = Rc::new(RadioGroup { name, value });
    let group_1 = group.clone();
    with_context::<_, RadioGroup<E>>(children, group)
        .or(async {
            loop {
                group_1.value.as_observable().until_change().await;
                on_change
                    .as_mut()
                    .map(|f| f(group_1.value.as_observable().borrow_observable_as().clone()));
            }
        })
        .await;
}

pub async fn radio_button<E: Clone + PartialEq + 'static>(RadioProps { value }: RadioProps<E>) {
    let ctx = get_context::<RadioGroup<E>>();
    let elem: HtmlInputElement = DOCUMENT.with(|doc| {
        let elem = doc.create_element("input").expect("create element failed");
        elem.unchecked_into()
    });
    elem.set_type("radio");
    elem.set_name(&ctx.name);

    let ctx_1 = ctx.clone();
    let elem_1 = elem.clone();
    let value_1 = value.clone();
    let func: Closure<dyn Fn(Event)> = Closure::new(move |_ev: Event| {
        if elem_1.checked() && *ctx_1.value.as_observable().borrow_observable_as() != value_1 {
            ctx_1.value.set(value_1.clone());
        }
    });
    elem.set_onchange(Some(func.as_ref().unchecked_ref()));

    let elem_2 = elem.clone();

    ElementFuture::new(
        async {
            loop {
                elem_2.set_checked(*ctx.value.as_observable().borrow_observable_as() == value);
                ctx.value.as_observable().until_change().await;
            }
        },
        elem.into(),
    )
    .await;
}
