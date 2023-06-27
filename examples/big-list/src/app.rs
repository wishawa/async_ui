use std::cell::RefCell;

use async_ui_web::{components::Div, shortcut_traits::ShortcutClassList, NoChild, VirtualizedList};

pub async fn app() {
    let root = Div::new();
    root.add_class(style::wrapper);
    let divs = &RefCell::new(Vec::new());
    let list = VirtualizedList::new(
        &root.element,
        Div::new().element.into(),
        Div::new().element.into(),
        |index| async move {
            let div = divs.borrow_mut().pop().unwrap_or_else(Div::new);
            div.set_text_content(Some(&index.to_string()));
            let fut = div.render(NoChild);
            scopeguard::defer!(
                divs.borrow_mut().push(div);
            );
            fut.await
        },
    );
    list.set_num_items(100000);
    root.render(list.render()).await;
}
mod style {
    use async_ui_web::css;

    css!(
        "
.wrapper {
	height: 75vh;
	width: 24em;
	overflow: scroll;
}
	"
    );
}
