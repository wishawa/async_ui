use async_ui_reactive::singlethread::ReactiveRefCell;
use async_ui_utils::{join, race};

use crate::{element::Element, wrappers::portal::create_portal};

pub async fn hidable(is_visible: &ReactiveRefCell<bool>, children: Vec<Element<'_>>) {
    let (entrance, mut exit) = create_portal();
    let exit_fut = async {
        let mut visible = { *is_visible.borrow() };
        loop {
            while !visible {
                visible = *is_visible.borrow_next().await;
            }
            race(exit.render_borrowed(), async {
                while visible {
                    visible = *is_visible.borrow_next().await;
                }
            })
            .await;
        }
    };
    join(entrance.render(children), exit_fut).await;
}
