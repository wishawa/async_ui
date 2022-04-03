use async_ui_reactive::Rx;
use async_ui_utils::{join, race, vec_into};
use futures::StreamExt;

use super::super::{backend::Backend, element::Element, render::render_with_control};
use super::portal::create_portal;
pub async fn hidable<B: Backend>(is_visible: &Rx<bool>, children: Vec<Element<'_, B>>) {
    let (entrance, mut exit) = create_portal();
    let mut stream = is_visible.listen();
    let exit_fut = async {
        loop {
            if is_visible.get() {
                race(
                    render_with_control(vec_into![exit.to_element_borrowed()], None),
                    async {
                        loop {
                            if !is_visible.get() {
                                break;
                            }
                            stream.next().await;
                        }
                    },
                )
                .await;
            }
            stream.next().await;
        }
    };
    join(
        render_with_control(vec_into![entrance.to_element(children)], None),
        exit_fut,
    )
    .await;
}
