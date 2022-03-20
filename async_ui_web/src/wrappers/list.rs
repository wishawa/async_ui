use std::{collections::HashMap, hash::Hash};

use async_ui_reactive::SRefCell;
use async_ui_spawn::singlethread::{SpawnedFuture, TaskWrapper};

use crate::{
    control::element_control::{ElementControl, ELEMENT_CONTROL},
    create_portal,
    element::Element,
    PortalExit,
};

pub async fn list<'a, K: Eq + Hash + Clone>(children: &'a SRefCell<Vec<(K, Option<Element<'a>>)>>) {
    struct ChildTask<'a> {
        _entry_task: TaskWrapper<'a>,
        exit_task: Option<TaskWrapper<'a>>,
        exit_portal: PortalExit,
        index: usize,
    }
    unsafe fn exit_task_from_portal(
        exit_portal: &PortalExit,
        parent_control: &ElementControl,
        index: usize,
    ) -> TaskWrapper<'static> {
        let mut exit_elem: Element<'static> = exit_portal.carefully_clone().render().into();
        exit_elem.set_control(parent_control.nest(index));
        let mut exit_future: SpawnedFuture<'static> =
            SpawnedFuture::new(exit_elem.to_boxed_future());
        let exit_task = unsafe { exit_future.launch_and_get_task() };
        exit_task
    }
    let parent_control = ELEMENT_CONTROL.with(Clone::clone);

    let mut tasks: HashMap<K, ChildTask> = HashMap::new();
    let mut new_tasks: Vec<(K, ChildTask)> = Vec::new();

    let mut list = children.get_mut();
    loop {
        for (index, (k, v)) in list.iter_mut().enumerate() {
            if let Some(element) = v.take() {
                let (entry_portal, exit_portal) = create_portal();
                let entry_elem: Element = entry_portal.render(vec![element]).into();
                let mut entry_future = SpawnedFuture::new(entry_elem.to_boxed_future());
                let entry_task = unsafe { entry_future.launch_and_get_task() };
                new_tasks.push((
                    k.clone(),
                    ChildTask {
                        _entry_task: entry_task,
                        exit_task: None,
                        exit_portal,
                        index,
                    },
                ));
            } else if let Some((k, mut child)) = tasks.remove_entry(k) {
                if child.index != index {
                    child.index = index;
                    child.exit_task = None;
                }
                new_tasks.push((k.clone(), child));
            }
        }
        tasks.clear();
        tasks.extend(new_tasks.drain(..).map(|(k, mut child)| {
            if child.exit_task.is_none() {
                let exit_task = unsafe {
                    exit_task_from_portal(&child.exit_portal, &parent_control, child.index)
                };
                child.exit_task = Some(exit_task);
            }
            (k, child)
        }));
        std::mem::drop(list);
        list = children.get_next_mut().await;
    }
}
