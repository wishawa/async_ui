use std::{collections::HashMap, hash::Hash};

use async_ui_reactive::Rx;
use async_ui_spawn::wasm::{DynamicSpawnedTasksContainer, SpawnedFuture, TaskWrapper};
use futures::StreamExt;

use crate::{
    control::element_control::ELEMENT_CONTROL, create_portal, element::Element, PortalExit,
};

pub async fn list<'a, K: Eq + Hash + Clone>(children: &Rx<Vec<(K, Option<Element<'a>>)>>) {
    struct ChildTask {
        exit_portal: PortalExit,
        exit_task: Option<TaskWrapper<'static>>,
        index: usize,
    }

    let parent_control = ELEMENT_CONTROL.with(Clone::clone);

    let num_initial_children = children.visit(Vec::len);
    let entry_tasks_container = DynamicSpawnedTasksContainer::with_capacity(num_initial_children);
    let mut tasks: HashMap<K, ChildTask> = HashMap::with_capacity(num_initial_children);
    futures::pin_mut!(entry_tasks_container);
    let mut new_tasks: Vec<(K, ChildTask)> = Vec::new();
    let mut stream = children.listen();
    loop {
        children.visit_mut_silent(|children| {
            let added_iter = children
                .iter_mut()
                .enumerate()
                .filter_map(|(idx, (k, opt))| {
                    if let Some(elem) = opt.take() {
                        let (entry, exit) = create_portal();
                        new_tasks.push((
                            k.clone(),
                            ChildTask {
                                exit_portal: exit,
                                exit_task: None,
                                index: idx,
                            },
                        ));
                        let entry_elem = entry.to_element(vec![elem]);
                        let fut = entry_elem.to_boxed_future();
                        Some((k.clone(), fut))
                    } else {
                        if let Some((k, mut child)) = tasks.remove_entry(k) {
                            if child.index != idx {
                                child.index = idx;
                                child.exit_task = None;
                            }
                            new_tasks.push((k, child));
                        }
                        None
                    }
                });
            entry_tasks_container.launch_futures(added_iter);
            entry_tasks_container.remove_futures(tasks.keys());
            tasks.clear();
            tasks.extend(new_tasks.drain(..).map(|(k, mut child)| {
                if child.exit_task.is_none() {
                    let mut exit_elem = child.exit_portal.carefully_clone().to_element();
                    exit_elem.set_control(parent_control.nest(child.index));
                    let task = SpawnedFuture::new(exit_elem.to_boxed_future()).launch();
                    child.exit_task = Some(task);
                }
                (k, child)
            }));
        });
        stream.next().await;
    }
}
