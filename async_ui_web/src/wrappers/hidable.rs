use async_ui_reactive::SRefCell;

use crate::{element::Element, wrappers::portal::create_portal};

pub async fn hidable(is_visible: &SRefCell<bool>, children: Vec<Element<'_>>) {
    let (entrance, mut exit) = create_portal();
    let exit_fut = async {
        let mut visible = { *is_visible.get() };
        loop {
            while !visible {
                visible = *is_visible.get_next().await;
            }
            smol::future::race(exit.render_borrowed(), async {
                while visible {
                    visible = *is_visible.get_next().await;
                }
            })
            .await;
        }
    };
    smol::future::zip(entrance.render(children), exit_fut).await;
}
