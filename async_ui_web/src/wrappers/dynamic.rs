use std::future::pending;

use async_ui_reactive::singlethread::ReactiveRefCell;
use async_ui_utils::race;

use crate::{element::Element, render};

pub async fn dynamic(child: &ReactiveRefCell<Option<Element<'_>>>) {
    let mut elem = child.borrow_mut().take();
    loop {
        let moved = elem.take();
        race(
            async {
                if let Some(e) = moved {
                    render(vec![e]).await
                } else {
                    pending().await
                }
            },
            async {
                while elem.is_none() {
                    elem = child.borrow_next_mut().await.take();
                }
            },
        )
        .await;
    }
}
