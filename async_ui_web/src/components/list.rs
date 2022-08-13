use std::future::{Future, IntoFuture};

use async_task::Task;
use async_ui_core::{
    backend::BackendTrait,
    executor::spawn_local,
    list::{Change, ListModel, ListModelPrivateAPIs},
    vnode::{
        node_concrete::{RefNode, WithConcreteNode},
        VNodeTrait,
    },
};
use futures::pin_mut;
use im_rc::Vector;
use observables::{Observable, ObservableExt};
use scoped_async_spawn::{boxed::ScopeSafeBox, SpawnGuard};
use slab::Slab;
use web_sys::Node;

use crate::{backend::Backend, window::DOCUMENT};

pub struct List<'c, T: Clone, F: IntoFuture<Output = ()>> {
    pub data: &'c (dyn Observable<ListModel<T>> + 'c),
    pub render: &'c (dyn Fn(T) -> F + 'c),
}

impl<'c, T: Clone, F: IntoFuture<Output = ()>> IntoFuture for List<'c, T, F> {
    type Output = ();
    type IntoFuture = ScopeSafeBox<dyn Future<Output = ()> + 'c>;
    fn into_future(self) -> Self::IntoFuture {
        ScopeSafeBox::from_boxed(Box::new(async move {
            let parent_vnode = Backend::get_vnode_key().with(Clone::clone);
            let container_node: Node = DOCUMENT
                .with(|doc| doc.create_element("div").expect("create element failed"))
                .into();
            parent_vnode.add_child_node(container_node.clone(), Default::default());
            let mut last_version: u64 = 0;
            let mut tasks: Slab<Task<()>> = Slab::new();
            let guard = SpawnGuard::new();
            pin_mut!(guard);
            let mut nodes: Vector<usize> = Vector::new();
            let mut create_item_task = |fut: F::IntoFuture| {
                let reference_node: Node = DOCUMENT.with(|doc| doc.create_comment("")).into();
                let fut = {
                    WithConcreteNode::new(
                        fut,
                        RefNode::<Backend>::Sibling {
                            parent: container_node.clone(),
                            sibling: reference_node,
                        },
                    )
                };
                let fut = guard.as_mut().convert_future(fut);
                let task = spawn_local(fut);
                task
            };
            {
                let model = &*self.data.get_borrow();
                let start = model.underlying_vector();
                for item in start.iter() {
                    let fut = (self.render)(item.to_owned()).into_future();
                    let task = create_item_task(fut);
                    let task_id = tasks.insert(task);
                    nodes.push_back(task_id);
                }
            }
            loop {
                {
                    let model = &*self.data.get_borrow();
                    let model_priv = ListModelPrivateAPIs(model);
                    let changes = model_priv.changes_since_version(last_version);
                    for change in changes {
                        match change {
                            Change::Splice {
                                remove_range,
                                replace_with,
                            } => {
                                let n_items = ExactSizeIterator::len(remove_range);
                                let mut right = nodes.split_off(remove_range.start);
                                let new_right = right.split_off(n_items);
                                for to_drop in right.into_iter() {
                                    std::mem::drop(tasks.remove(to_drop));
                                }
                                nodes.extend(replace_with.iter().map(|t| {
                                    let fut = (self.render)(t.to_owned()).into_future();
                                    let task = create_item_task(fut);
                                    let task_id = tasks.insert(task);
                                    task_id
                                }));
                                nodes.append(new_right);
                            }
                            Change::Remove { index } => {
                                let task_id = nodes.remove(*index);
                                std::mem::drop(tasks.remove(task_id));
                            }
                            Change::Insert { index, value } => {
                                let fut = (self.render)(value.to_owned()).into_future();
                                let task = create_item_task(fut);
                                let task_id = tasks.insert(task);
                                nodes.insert(*index, task_id);
                            }
                        }
                    }
                    last_version = model_priv.get_version();
                    model_priv
                        .pending_listeners()
                        .set(model_priv.pending_listeners().get() - 1);
                }
                self.data.until_change().await;
            }
        }))
    }
}
