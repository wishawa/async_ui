use std::{cell::RefCell, collections::BTreeMap};

use wasm_bindgen::UnwrapThrowExt;

use crate::position::ChildPosition;

pub(crate) enum DomContext<'p> {
    Container {
        group: &'p NodeGroup,
        container: &'p web_sys::Node,
    },
    Sibling {
        parent: &'p Self,
        group: &'p NodeGroup,
        reference: &'p web_sys::Node,
    },
    Child {
        parent: &'p Self,
        index: u32,
    },
    #[cfg(test)]
    Null,
}

scoped_tls_hkt::scoped_thread_local!(
    pub(crate) static DOM_CONTEXT: for<'p> &'p DomContext<'p>
);

pub(crate) type NodeGroup = RefCell<BTreeMap<ChildPosition, web_sys::Node>>;

pub fn get_containing_node() -> web_sys::Node {
    DOM_CONTEXT.with(|ctx: &DomContext| ctx.get_containing_node().to_owned())
}

impl<'p> DomContext<'p> {
    pub fn get_containing_node(&self) -> &web_sys::Node {
        match self {
            DomContext::Container { container, .. } => container,
            DomContext::Child { parent, .. } | DomContext::Sibling { parent, .. } => {
                parent.get_containing_node()
            }
            #[cfg(test)]
            DomContext::Null => unreachable!(),
        }
    }
    /// Add a new node `new_child` ordered relative to existing siblings according to `position`.
    pub fn add_child(&self, mut position: ChildPosition, new_child: web_sys::Node) {
        match self {
            DomContext::Container { group, container } => {
                let mut group = group.borrow_mut();
                let reference_sibling = group.range((&position)..).next().map(|(_k, v)| v);
                container
                    .insert_before(&new_child, reference_sibling)
                    .unwrap_throw();
                group.insert(position, new_child);
            }
            DomContext::Sibling {
                parent,
                group,
                reference,
            } => {
                let mut group = group.borrow_mut();
                let reference_sibling = group
                    .range((&position)..)
                    .next()
                    .map(|(_k, v)| v)
                    .unwrap_or(*reference);
                parent
                    .get_containing_node()
                    .insert_before(&new_child, Some(reference_sibling))
                    .unwrap_throw();
                group.insert(position, new_child);
            }
            DomContext::Child { parent, index } => {
                position.wrap(*index);
                parent.add_child(position, new_child);
            }
            #[cfg(test)]
            DomContext::Null => {}
        }
    }
    /// Remove the child at `position` and all its descendants.
    pub fn remove_child(&self, mut position: ChildPosition) {
        match self {
            DomContext::Container { group, container } => {
                let mut group = group.borrow_mut();
                remove_children_here(&mut group, position, container);
            }
            DomContext::Sibling { group, parent, .. } => {
                let mut group = group.borrow_mut();
                remove_children_here(&mut group, position, parent.get_containing_node());
            }
            DomContext::Child { parent, index } => {
                position.wrap(*index);
                parent.remove_child(position);
            }
            #[cfg(test)]
            DomContext::Null => {}
        }
    }
}

fn remove_children_here(
    tree: &mut BTreeMap<ChildPosition, web_sys::Node>,
    position: ChildPosition,
    container: &web_sys::Node,
) {
    if position.is_root() {
        tree.values().for_each(|child| {
            container.remove_child(child).unwrap_throw();
        });
        tree.clear();
    } else {
        let next = position.next_sibling();
        let range = (&position)..(&next);
        while let Some((key, child)) = tree.range(range.clone()).next_back() {
            container.remove_child(child).unwrap_throw();
            tree.remove(&key.clone());
        }
    }
}
