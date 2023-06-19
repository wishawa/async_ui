use async_ui_web::{
    components::Div,
    shortcut_traits::{ShortcutClassList, ShortcutRenderStr},
    VirtualizedList,
};

pub async fn app() {
    let root = Div::new();
    root.add_class(style::wrapper);
    let list = VirtualizedList::new(
        &root.element,
        Div::new().element.into(),
        Div::new().element.into(),
        |recycle, index| Div::new().render(index.to_string().render()),
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
