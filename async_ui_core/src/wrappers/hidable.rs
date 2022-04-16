use async_ui_reactive::local::Rx;
use async_ui_utils::{Join, Race};
use futures::StreamExt;

use crate::tuple::TupleOfFutures;

use super::super::backend::Backend;
use super::portal::create_portal;
pub async fn hidable<'e, B: Backend, C: TupleOfFutures<'e>>(is_visible: &Rx<bool>, children: C) {
    let (entrance, mut exit) = create_portal::<B>();
    let mut stream = is_visible.listen();
    let exit_fut = async {
        loop {
            if is_visible.get() {
                Race::from((exit.render_borrowed(), async {
                    loop {
                        if !is_visible.get() {
                            break;
                        }
                        stream.next().await;
                    }
                }))
                .await;
            }
            stream.next().await;
        }
    };
    Join::from((entrance.render(children), exit_fut)).await;
}
