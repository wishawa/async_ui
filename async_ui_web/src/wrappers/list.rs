use std::{collections::HashMap, hash::Hash};

use async_ui_core::{
    backend::{Backend, Spawner},
    drop_check::check_drop_scope,
    render::spawn_with_control,
};
use async_ui_reactive::Rx;
use async_ui_spawners::web::WebSpawner;
use futures::StreamExt;

use crate::{backend::WebBackend, create_portal, element::Element, PortalExit};

pub async fn list<'a, K: Eq + Hash + Clone>(children: &Rx<Vec<(K, Option<Element<'a>>)>>) {
    struct ChildTask {
        exit_portal: PortalExit,
        exit_task: Option<<WebSpawner as Spawner>::Task>,
        _entry_task: <WebSpawner as Spawner>::Task,
        index: usize,
    }
    let parent_control = WebBackend::get_tls().with(Clone::clone);
    check_drop_scope(&parent_control as *const _ as *const ());

    let num_initial_children = children.visit(Vec::len);
    let mut tasks: HashMap<K, ChildTask> = HashMap::with_capacity(num_initial_children);
    let mut new_tasks: Vec<(K, ChildTask)> = Vec::new();
    let mut stream = children.listen();
    loop {
        children.visit_mut_silent(|children| {
            children.iter_mut().enumerate().for_each(|(idx, (k, opt))| {
                if let Some(elem) = opt.take() {
                    let (entry, exit) = create_portal();
                    let entry_task =
                        unsafe { spawn_with_control(entry.to_element(vec![elem]), None) };
                    new_tasks.push((
                        k.clone(),
                        ChildTask {
                            exit_portal: exit,
                            exit_task: None,
                            _entry_task: entry_task,
                            index: idx,
                        },
                    ))
                } else {
                    if let Some((k, mut child)) = tasks.remove_entry(&k) {
                        if child.index != idx {
                            child.index = idx;
                            child.exit_task = None;
                        }
                        new_tasks.push((k, child));
                    }
                }
            });
            tasks.clear();
            tasks.extend(new_tasks.drain(..).map(|(k, mut child)| {
                if child.exit_task.is_none() {
                    let exit_elem = child.exit_portal.carefully_clone().to_element();
                    let control = parent_control.nest(child.index);
                    let task = unsafe { spawn_with_control(exit_elem, Some(control)) };
                    child.exit_task = Some(task);
                }
                (k, child)
            }));
        });
        stream.next().await;
    }
}
