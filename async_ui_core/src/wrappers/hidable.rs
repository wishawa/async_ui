use async_ui_reactive::Rx;
use async_ui_utils::{join2, race2};
use futures::StreamExt;

use crate::tuple::TupleOfFutures;

use super::super::backend::Backend;
use super::portal::create_portal;
pub async fn hidable<'e, B: Backend, C: TupleOfFutures<'e, B>>(is_visible: &Rx<bool>, children: C) {
    let (entrance, mut exit) = create_portal();
    let mut stream = is_visible.listen();
    let exit_fut = async {
        loop {
            if is_visible.get() {
                race2(exit.render_borrowed(), async {
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
    join2(entrance.render(children), exit_fut).await;
}
