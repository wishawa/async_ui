use std::{cell::Cell, cell::RefCell, collections::HashMap, ops::Range, rc::Rc};

use async_ui_gtk::{
    manual_apis::{control_from_node, ContainerHandler},
    ManyRender, ManyRenderKey, Render,
};
use async_ui_reactive::local::{unbounded, Rx};
use async_ui_utils::Join;
use futures::pin_mut;
use glib::{Cast, ObjectExt, StaticType};
use gtk::{
    gio::{prelude::ListModelExt, ListStore},
    glib, ListItem, SignalListItemFactory, SingleSelection,
};

use crate::Wrappable;

glib::wrapper! {
    struct KeyObject(ObjectSubclass<imp::KeyObject>);
}
impl KeyObject {
    fn new(key: u32) -> Self {
        glib::Object::new(&[("key", &key)]).unwrap()
    }
    fn get_key_id(&self) -> u32 {
        self.property("key")
    }
}
mod imp {
    use glib::{
        once_cell::sync::Lazy,
        subclass::{object::ObjectImpl, types::ObjectSubclass},
        ParamFlags, ParamSpec, ParamSpecUInt, ToValue, Value,
    };
    use gtk::glib;
    use std::cell::Cell;

    #[derive(Default)]
    pub(super) struct KeyObject {
        key: Cell<u32>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for KeyObject {
        const NAME: &'static str = "KeyObject";
        type Type = super::KeyObject;
        type ParentType = glib::Object;
    }
    impl ObjectImpl for KeyObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecUInt::new(
                    "key",
                    "key",
                    "key",
                    0,
                    u32::MAX,
                    0,
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }
        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "key" => {
                    let key: u32 = value.get().expect("The value needs to be of type `u32`.");
                    self.key.replace(key);
                }
                _ => unimplemented!(),
            }
        }
        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "key" => self.key.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

struct ListItemHandler;
impl ContainerHandler for ListItemHandler {
    fn get_support_multichild(&self) -> bool {
        false
    }
    fn set_single_child(&self, this: &glib::Object, child: Option<&gtk::Widget>) {
        let item: &ListItem = this.downcast_ref().unwrap();
        item.set_child(child);
    }
}

pub struct ListViewItems<K> {
    store: ListStore,
    keys_map: RefCell<HashMap<u32, K>>,
    top_id: Cell<u32>,
}
impl<K> ListViewItems<K> {
    pub fn new() -> Self {
        let store = ListStore::new(KeyObject::static_type());
        let top_id = Cell::new(0);
        let keys_map = RefCell::new(HashMap::new());
        Self {
            store,
            keys_map,
            top_id,
        }
    }
    pub fn splice<I: Iterator<Item = K>>(&self, range: Range<usize>, replace_with: I) {
        let start = range.start;
        let end = range.end;
        let del_num = end - start;
        let mut ids = Vec::with_capacity(replace_with.size_hint().0);
        let mut keys_map = self.keys_map.borrow_mut();
        range.for_each(|i| {
            if let Some(item) = self.store.item(i as u32) {
                let key_obj: &KeyObject = item.downcast_ref().unwrap();
                let key_id = key_obj.get_key_id();
                keys_map.remove(&key_id);
            }
        });
        let last_id = self.top_id.get();
        keys_map.extend(replace_with.enumerate().map(|(idx, k)| {
            let key_id = last_id + idx as u32;
            ids.push(KeyObject::new(key_id));
            (key_id, k)
        }));
        self.top_id.set(last_id + ids.len() as u32);
        self.store.splice(start as u32, del_num as u32, &ids);
    }
    pub fn append(&self, item: K) {
        let last_id = self.top_id.get();
        self.keys_map.borrow_mut().insert(last_id, item);
        self.top_id.set(last_id + 1);
        self.store.append(&KeyObject::new(last_id));
    }
    pub fn remove(&self, position: usize) {
        self.splice(position..(position + 1), [].into_iter());
    }
    pub fn insert(&self, position: usize, item: K) {
        let last_id = self.top_id.get();
        self.keys_map.borrow_mut().insert(last_id, item);
        self.store.insert(position as u32, &KeyObject::new(last_id));
        self.top_id.set(last_id + 1);
    }
}

pub async fn list_view<'a, K, F: Fn(Rc<Rx<K>>) -> Render<'a>>(
    items: &ListViewItems<K>,
    factory: F,
) {
    enum BindCommand {
        Bind(u32, ListItem),
        Teardown(ListItem),
    }
    struct Item<K> {
        key: Rc<Rx<K>>,
        key_id: Cell<u32>,
        render_key: Cell<ManyRenderKey>,
    }
    let mut items_map: HashMap<ListItem, Item<K>> = HashMap::new();

    let renders = ManyRender::with_capacity(0);
    pin_mut!(renders);

    let signal = SignalListItemFactory::new();
    let (tx, rx) = unbounded();
    let bind_tx = tx.clone();
    signal.connect_bind(move |_sig, list_item| {
        println!("binding");
        let item = list_item.item().expect("item is null");
        let key: &KeyObject = item.downcast_ref().expect("item downcast failed");
        let key_id = key.get_key_id();
        let list_item = list_item.to_owned();
        bind_tx.send(BindCommand::Bind(key_id, list_item));
    });
    let teardown_tx = tx;
    signal.connect_teardown(move |_sig, list_item| {
        println!("tearing down");
        teardown_tx.send(BindCommand::Teardown(list_item.to_owned()));
    });
    let rx_fut = rx.for_each(|command| match command {
        BindCommand::Bind(key_id, list_item) => {
            let mut keys_map = items.keys_map.borrow_mut();
            let item = items_map.get_mut(&list_item);
            if let Some(item) = item {
                let old_key_id = item.key_id.replace(key_id);
                if old_key_id != key_id {
                    let key = keys_map.remove(&key_id).expect("key already used");
                    let old_key = item.key.visit_mut(|inn| std::mem::replace(inn, key));
                    keys_map.insert(old_key_id, old_key);
                }
            } else {
                let key = keys_map.remove(&key_id).expect("key already used");
                let key = Rc::new(Rx::new(key));
                let control = control_from_node(list_item.clone().upcast(), &ListItemHandler);
                let render_key = renders.as_mut().add_render(factory(key.clone()), control);
                let render_key = Cell::new(render_key);
                let key_id = Cell::new(key_id);
                items_map.insert(
                    list_item,
                    Item {
                        key,
                        key_id,
                        render_key,
                    },
                );
            }
        }
        BindCommand::Teardown(list_item) => {
            let item = items_map.remove(&list_item);
            if let Some(item) = item {
                renders.as_mut().remove_render(item.render_key.get());
                if let Ok(inner) = Rc::try_unwrap(item.key) {
                    let key = inner.into_inner();
                    let mut keys_map = items.keys_map.borrow_mut();
                    keys_map.insert(item.key_id.get(), key);
                } else {
                    panic!("key not dropped");
                }
            }
        }
    });
    let selection_model = SingleSelection::new(Some(&items.store));
    let render_fut =
        Render::from((gtk::ListView::new(Some(&selection_model), Some(&signal)).wrap(),));
    Join::from((rx_fut, render_fut)).await;
}
