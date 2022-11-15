use std::{
    collections::{HashMap, VecDeque},
    future::IntoFuture,
    rc::Rc,
};

use async_task::Task;
pub use async_ui_core::list::ListModel;
use async_ui_core::{
    backend::BackendTrait,
    executor::spawn_local,
    list::{Change, ListModelPrivateAPIs},
    vnode::{
        node_concrete::{ConcreteNodeVNode, RefNode},
        VNodeTrait, WithVNode,
    },
};
use futures_lite::pin;
use glib::{Cast, Object, ObjectExt, StaticType};
use gtk::{gio::ListStore, NoSelection, Widget};
use im_rc::Vector;
use observables::{cell::ReactiveCell, ObservableAs, ObservableAsExt};
use scoped_async_spawn::SpawnGuard;

use crate::{
    backend::Backend,
    widget::{SingleChildWidgetOp, WidgetOp, WrappedWidget},
};

use super::ElementFuture;

glib::wrapper! {
    pub struct KeyObject(ObjectSubclass<imp::KeyObject>);
}

mod imp {
    use std::cell::Cell;

    use glib::{
        subclass::{prelude::ObjectImpl, types::ObjectSubclass},
        ParamSpec, ToValue, Value,
    };
    use gtk::glib::{once_cell::sync::Lazy, ParamFlags, ParamSpecUInt};

    #[derive(Default)]
    pub struct KeyObject {
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
impl KeyObject {
    pub fn new(id: u32) -> Self {
        Object::new(&[("key", &id)]).expect("ListItemId creation failed")
    }
    fn get_key_id(&self) -> u32 {
        self.property("key")
    }
}

pub struct ListProps<'c, T: Clone, F: IntoFuture> {
    pub data: Option<&'c dyn ObservableAs<ListModel<T>>>,
    pub render: Option<&'c dyn Fn(T) -> F>,
}
impl<'c, T: Clone, F: IntoFuture> Default for ListProps<'c, T, F> {
    fn default() -> Self {
        Self {
            data: None,
            render: None,
        }
    }
}
struct ListItemWidgetOp;
impl SingleChildWidgetOp for ListItemWidgetOp {
    fn set_child(&self, this: &Object, child: &mut WrappedWidget) {
        let this = this.downcast_ref::<gtk::ListItem>().unwrap();
        this.set_child(Some(&child.widget))
    }

    fn get_child(&self, this: &Object) -> Option<Widget> {
        let this = this.downcast_ref::<gtk::ListItem>().unwrap();
        this.child()
    }

    fn unset_child(&self, this: &Object) {
        let this = this.downcast_ref::<gtk::ListItem>().unwrap();
        this.set_child(Option::<&Widget>::None)
    }
}

pub async fn list<'c, T: Clone, F: IntoFuture>(ListProps { data, render }: ListProps<'c, T, F>) {
    #[derive(Debug)]
    struct ItemAndTask<T> {
        item: T,
        task: Option<Task<()>>,
    }
    struct BindCommand {
        list_item: gtk::ListItem,
        key: u32,
        is_bind: bool,
    }
    let (data, render) = match (data, render) {
        (Some(d), Some(r)) => (d, r),
        _ => {
            return;
        }
    };
    let store = ListStore::new(KeyObject::static_type());
    let selection_model = NoSelection::new(Some(&store));
    let factory = gtk::SignalListItemFactory::new();
    let list_view = gtk::ListView::new(Some(&selection_model), Some(&factory));
    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&list_view)
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .build();
    let dummy_widget = gtk::Label::new(None);

    let dummy_widget_copy = dummy_widget.clone();
    let inside = async move {
        let mut current_id = 0;
        let start_bm = data.borrow_observable_as();
        let start = start_bm.underlying_vector();
        let mut keys_map: HashMap<u32, ItemAndTask<T>> = HashMap::with_capacity(start.len());
        let mut keys_list: Vector<u32> = Vector::new();

        let parent_vnode = Backend::get_vnode_key().with(Clone::clone);
        let parent_context = parent_vnode.get_context_map();

        let guard = SpawnGuard::new();
        pin!(guard);
        let bind_channel = Rc::new(ReactiveCell::new(VecDeque::new()));

        let bind_channel_bind = bind_channel.clone();
        factory.connect_bind(move |_fac, li| {
            let position = li.position();
            let key = li
                .item()
                .unwrap()
                .downcast::<KeyObject>()
                .unwrap()
                .get_key_id();
            println!("send bind {key} @ {position}");
            bind_channel_bind.borrow_mut().push_back(BindCommand {
                list_item: li.to_owned(),
                key,
                is_bind: true,
            });
            glib::MainContext::default().iteration(false);
        });
        let bind_channel_unbind = bind_channel.clone();
        factory.connect_unbind(move |_fac, li| {
            let position = li.position();
            let key = li
                .item()
                .unwrap()
                .downcast::<KeyObject>()
                .unwrap()
                .get_key_id();
            println!("send unbind {key} @ {position}");
            bind_channel_unbind.borrow_mut().push_back(BindCommand {
                list_item: li.to_owned(),
                key,
                is_bind: false,
            });
            glib::MainContext::default().iteration(false);
        });

        let mut last_version = {
            {
                for item in start.iter() {
                    current_id += 1;
                    keys_map.insert(
                        current_id,
                        ItemAndTask {
                            item: item.to_owned(),
                            task: None,
                        },
                    );
                    keys_list.push_back(current_id);
                    store.append(&KeyObject::new(current_id));
                }
            }
            let model = ListModelPrivateAPIs(&start_bm);
            model
                .total_listeners()
                .set(model.total_listeners().get() + 1);
            model.get_version()
        };
        drop(start_bm);
        let _guard = scopeguard::guard((), |_| {
            let b = data.borrow_observable_as();
            let model = ListModelPrivateAPIs(&*b);
            model
                .total_listeners()
                .set(model.total_listeners().get() - 1);
        });

        loop {
            {
                println!("reading changes");
                let model = &*data.borrow_observable_as();
                let model_priv = ListModelPrivateAPIs(model);
                let changes = model_priv.changes_since_version(last_version);
                let current_version = model_priv.get_version();
                if last_version != current_version {
                    println!("there's change");
                    for change in changes {
                        match change {
                            Change::Splice {
                                remove_range,
                                replace_with,
                            } => {
                                let n_items = ExactSizeIterator::len(remove_range);
                                let mut right = keys_list.split_off(remove_range.start);
                                let mut new_right = right.split_off(n_items);
                                for key in right.into_iter() {
                                    store.remove(remove_range.start as u32);
                                    keys_map.remove(&key);
                                }
                                for item in replace_with.iter().rev() {
                                    current_id += 1;
                                    keys_map.insert(
                                        current_id,
                                        ItemAndTask {
                                            item: item.to_owned(),
                                            task: None,
                                        },
                                    );
                                    new_right.push_front(current_id);
                                    store.insert(
                                        remove_range.start as u32,
                                        &KeyObject::new(current_id),
                                    );
                                }
                                keys_list.append(new_right);
                            }
                            Change::Remove { index } => {
                                let key = keys_list.remove(*index);
                                store.remove(*index as u32);
                                keys_map.remove(&key).unwrap();
                            }
                            Change::Insert { index, value } => {
                                current_id += 1;
                                println!("insert {current_id}");
                                keys_map.insert(
                                    current_id,
                                    ItemAndTask {
                                        item: value.to_owned(),
                                        task: None,
                                    },
                                );
                                keys_list.insert(*index, current_id);
                                store.insert(*index as u32, &KeyObject::new(current_id));
                            }
                        }
                    }
                    last_version = current_version;
                    model_priv
                        .pending_listeners()
                        .set(model_priv.pending_listeners().get() - 1);
                }
            }
            let data_fut = data.until_change();
            {
                for BindCommand {
                    list_item,
                    key,
                    is_bind,
                } in bind_channel.borrow_mut().drain(..)
                {
                    match (keys_map.get_mut(&key), is_bind) {
                        (Some(ItemAndTask { item, task }), true) => {
                            println!("bind {key} @");
                            if task.is_none() {
                                let fut = render(item.to_owned()).into_future();
                                let vnode = ConcreteNodeVNode::<Backend>::new(
                                    RefNode::Parent {
                                        parent: WrappedWidget {
                                            widget: dummy_widget_copy.clone().upcast(),
                                            inner_widget: list_item.to_owned().upcast(),
                                            op: WidgetOp::SingleChild(&ListItemWidgetOp),
                                        },
                                    },
                                    parent_context.clone(),
                                );
                                let fut = WithVNode::<Backend, _>::new(
                                    async {
                                        fut.await;
                                    },
                                    Rc::new(vnode.into()),
                                );
                                let fut = guard.as_mut().convert_future(fut);
                                *task = Some(spawn_local(fut));
                            } else {
                                panic!("double bind {key} @",);
                            }
                        }
                        (Some(ItemAndTask { task, .. }), false) => {
                            println!("unbind {key} @");
                            if task.is_some() {
                                *task = None;
                                list_item.set_child(Option::<&gtk::Widget>::None);
                            } else {
                                panic!("double unbind {key} @");
                            }
                        }
                        _ => {}
                    }
                }
            }

            use futures_lite::FutureExt;
            bind_channel
                .as_observable()
                .until_change()
                .or(data_fut)
                .await;
        }
    };

    ElementFuture::new(
        inside,
        WrappedWidget {
            widget: scrolled_window.clone().upcast(),
            inner_widget: dummy_widget.upcast(),
            op: WidgetOp::NoChild,
        },
    )
    .await;
}
