use std::{collections::HashMap, hash::Hash};

use crate::control::Control;
use crate::render::Render;

use super::super::{backend::Backend, drop_check::check_drop_scope, element::Element};
use super::portal::{create_portal, PortalExit};
use async_ui_reactive::local::Rx;
use futures::StreamExt;
struct ChildTask<'e, B: Backend> {
	exit_portal: PortalExit<B>,
	exit_task: Option<Element<'e, B>>,
	_entry_task: Element<'e, B>,
	index: usize,
}

pub async fn list_by_renders<'a, B: Backend, K: Eq + Hash + Clone>(
	children: &Rx<Vec<(K, Option<Render<'a, B>>)>>,
) {
	let parent_control = B::get_tls().with(Clone::clone);
	check_drop_scope(&parent_control as *const _ as *const ());

	let num_initial_children = children.visit(Vec::len);
	let mut tasks: HashMap<K, ChildTask<B>> = HashMap::with_capacity(num_initial_children);
	let mut new_tasks: Vec<(K, ChildTask<B>)> = Vec::new();
	let mut stream = children.listen();
	loop {
		children.visit_mut_silent(|children| {
			children.iter_mut().enumerate().for_each(|(idx, (k, opt))| {
				if let Some(elem) = opt.take() {
					let child = unsafe { create_child_task(elem, idx) };
					new_tasks.push((k.clone(), child));
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
			unsafe { update_tasks(&mut tasks, &mut new_tasks, &parent_control) };
		});
		stream.next().await;
	}
}

pub async fn list<'a, B: Backend, K: Eq + Hash + Clone, F: FnMut(&K) -> Render<'a, B>>(
	children: &Rx<Vec<K>>,
	mut factory: F,
) {
	let parent_control = B::get_tls().with(Clone::clone);
	check_drop_scope(&parent_control as *const _ as *const ());

	let num_initial_children = children.visit(Vec::len);
	let mut tasks: HashMap<K, ChildTask<B>> = HashMap::with_capacity(num_initial_children);
	let mut new_tasks: Vec<(K, ChildTask<B>)> = Vec::new();
	children
		.for_each(|children| {
			children.iter().enumerate().for_each(|(idx, k)| {
				if let Some((k, mut child)) = tasks.remove_entry(k) {
					if child.index != idx {
						child.index = idx;
						child.exit_task = None;
					}
					new_tasks.push((k, child));
				} else {
					let child = unsafe { create_child_task(factory(k), idx) };
					new_tasks.push((k.clone(), child));
				}
			});
			unsafe { update_tasks(&mut tasks, &mut new_tasks, &parent_control) };
		})
		.await;
}

unsafe fn create_child_task<'a, B: Backend>(render: Render<'a, B>, idx: usize) -> ChildTask<'a, B> {
	let (entry, exit) = create_portal();
	let entry_render = entry.render(render);
	let mut entry_task: Element<'a, B> = entry_render.into();
	unsafe { entry_task.mount(B::get_dummy_control()) };
	ChildTask {
		_entry_task: entry_task,
		exit_portal: exit,
		exit_task: None,
		index: idx,
	}
}
unsafe fn update_tasks<'a, B: Backend, K: Eq + Clone + Hash>(
	tasks: &mut HashMap<K, ChildTask<'a, B>>,
	new_tasks: &mut Vec<(K, ChildTask<'a, B>)>,
	parent_control: &'a Control<B>,
) {
	tasks.clear();
	tasks.extend(new_tasks.drain(..).map(|(k, mut child)| {
		if child.exit_task.is_none() {
			let exit_render = child.exit_portal.carefully_clone().render();
			let mut exit_task: Element<'static, B> = exit_render.into();
			let control = parent_control.nest(child.index);
			unsafe { exit_task.mount(control) };
			child.exit_task = Some(exit_task);
		}
		(k, child)
	}));
}
