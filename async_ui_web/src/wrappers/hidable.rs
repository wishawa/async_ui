use async_ui_reactive::Rx;
use async_ui_utils::{join, race, vec_into};
use futures::StreamExt;

use crate::{element::Element, render::render, wrappers::portal::create_portal};

pub async fn hidable(is_visible: &Rx<bool>, children: Vec<Element<'_>>) {
    let (entrance, mut exit) = create_portal();
    let mut stream = is_visible.listen();
    let exit_fut = async {
        loop {
            if is_visible.get() {
                race(render(vec_into![exit.to_element_borrowed()]), async {
                    loop {
                        if !is_visible.get() {
                            break;
                        }
                        stream.next().await;
                    }
                })
                .await;
            }
            stream.next().await;
        }
    };
    join(render(vec_into![entrance.to_element(children)]), exit_fut).await;
}
