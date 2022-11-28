use std::{future::IntoFuture, marker::PhantomData, rc::Rc};

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
use async_ui_props::dummy::DummyObservableAs;
use futures_lite::pin;
use im_rc::Vector;
use observables::{ObservableAs, ObservableAsExt};
use scoped_async_spawn::SpawnGuard;
use slab::Slab;
use web_sys::Node;

use crate::{backend::Backend, utils::class_list::ClassList, window::DOCUMENT};

use super::ElementFuture;

fn insert_after(parent: &Node, child: &Node, after: Option<&Node>) {
    let before = after.map_or_else(|| parent.first_child(), |after| after.next_sibling());
    parent
        .insert_before(child, before.as_ref())
        .expect("insert failed");
}

pub struct ListProps<'c, T: Clone, F: IntoFuture> {
    pub data: &'c dyn ObservableAs<ListModel<T>>,
    pub render: &'c dyn Fn(T) -> F,
    pub class: Option<&'c ClassList<'c>>,
}

impl<'c, T: Clone + 'c, F: IntoFuture> Default for ListProps<'c, T, F> {
    fn default() -> Self {
        Self {
            data: &DummyObservableAs(PhantomData),
            render: &|_: T| panic!("render function not provided"),
            class: None,
        }
    }
}

pub async fn list<'c, T: Clone + 'c, F: IntoFuture + 'c>(
    ListProps {
        data,
        render,
        class,
    }: ListProps<'c, T, F>,
) {
    let container_node =
        DOCUMENT.with(|doc| doc.create_element("div").expect("create element failed"));

    class.map(|c| c.set_dom(container_node.class_list()));

    let container_node: Node = container_node.clone().into();
    let container_node_copy = container_node.clone();
    let inside = async move {
        let parent_vnode = Backend::get_vnode_key().with(Clone::clone);

        let parent_context = parent_vnode.get_context_map();
        let mut tasks: Slab<Task<()>> = Slab::new();
        let guard = SpawnGuard::new();
        pin!(guard);
        let mut nodes: Vector<(Node, usize)> = Vector::new();
        let mut create_item_task = |fut: F::IntoFuture, after_this: Option<&Node>| {
            let reference_node: Node = DOCUMENT.with(|doc| doc.create_comment("")).into();
            insert_after(&container_node, &reference_node, after_this);
            let fut = {
                WithVNode::new(
                    fut,
                    Rc::new(
                        ConcreteNodeVNode::new(
                            RefNode::<Backend>::Sibling {
                                parent: container_node.clone(),
                                sibling: reference_node.clone(),
                            },
                            parent_context.clone(),
                        )
                        .into(),
                    ),
                )
            };
            let fut = guard.as_mut().convert_future(async {
                fut.await;
            });
            let task = spawn_local(fut);
            (reference_node, task)
        };
        let mut last_version = data.visit(|model| {
            let start = model.underlying_vector();
            let mut last_node = None;
            for item in start.iter() {
                let fut = render(item.to_owned()).into_future();
                let (node, task) = create_item_task(fut, last_node.as_ref());
                last_node = Some(node.to_owned());
                let task_id = tasks.insert(task);
                nodes.push_back((node, task_id));
            }
            let model = ListModelPrivateAPIs(model);
            model
                .total_listeners()
                .set(model.total_listeners().get() + 1);
            model.get_version()
        });
        let _guard = scopeguard::guard((), |_| {
            data.visit(|model| {
                let model = ListModelPrivateAPIs(&*model);
                model
                    .total_listeners()
                    .set(model.total_listeners().get() - 1);
            });
        });
        let mut first = true;
        data.for_each(|model| {
            if first {
                first = false;
                return;
            }
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
                        for (node, task_id) in right.into_iter() {
                            std::mem::drop(tasks.remove(task_id));
                            container_node.remove_child(&node).ok();
                        }
                        let insert_after: Option<Node> =
                            nodes.back().map(|(node, _)| node).cloned();
                        nodes.extend(replace_with.iter().map(|t| {
                            let fut = render(t.to_owned()).into_future();
                            let (node, task) = create_item_task(fut, insert_after.as_ref());
                            let task_id = tasks.insert(task);
                            (node, task_id)
                        }));
                        nodes.append(new_right);
                    }
                    Change::Remove { index } => {
                        let (node, task_id) = nodes.remove(*index);
                        std::mem::drop(tasks.remove(task_id));
                        container_node.remove_child(&node).ok();
                    }
                    Change::Insert { index, value } => {
                        let fut = render(value.to_owned()).into_future();
                        let (node, task) = create_item_task(fut, {
                            (*index > 0)
                                .then(|| nodes.get(index - 1).map(|(node, _task_id)| node))
                                .flatten()
                        });
                        let task_id = tasks.insert(task);
                        nodes.insert(*index, (node, task_id));
                    }
                }
            }
            last_version = model_priv.get_version();
            model_priv
                .pending_listeners()
                .set(model_priv.pending_listeners().get() - 1);
        })
        .await;
    };
    ElementFuture::new(inside, container_node_copy).await
}
