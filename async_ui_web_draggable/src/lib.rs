use std::{future::Future, hash::Hash};

use async_ui_web::lists::ModeledList;
use futures_lite::future::BoxedLocal;
use web_sys::HtmlDivElement;

// struct Shared {
//     placeholder: HtmlDivElement,
//     wrappers: Vec<HtmlDivElement>
// }

// pub struct ItemInfo<K> {
//     key: K,
// }

// impl<K> ItemInfo<K> {}

// pub struct DraggableList<'c, K: Eq + Hash + Clone> {
//     list: ModeledList<'c, K, BoxedLocal<()>, Box<dyn Fn(&K) -> BoxedLocal<()>>>,
// }

// impl<'c, K: Eq + Hash + Clone> DraggableList<'c, K> {
//     pub fn new<F: Future, R: Fn(ItemInfo<K>) -> F>(renderer: R) -> Self {
//         Self {
//             list: ModeledList::new({

//                 move |k| async move {}
//         }),
//         }
//     }
// }
