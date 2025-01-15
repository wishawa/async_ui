use std::{cell::RefCell, collections::BTreeMap};

use wasm_bindgen::UnwrapThrowExt;

use crate::dom::Node;
use crate::position::ChildPosition;

pub(crate) enum DomContext<'p> {
    Container {
        group: &'p NodeGroup,
        container: &'p Node,
    },
    Sibling {
        parent: &'p Self,
        group: &'p NodeGroup,
        reference: &'p Node,
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

pub(crate) type NodeGroup = RefCell<BTreeMap<ChildPosition, Node>>;

impl<'p> DomContext<'p> {
    /// Get the HTML node where the current code would render in.
    /// This is used by [SiblingNodeFuture][crate::SiblingNodeFuture] to decide where to add children.
    pub fn get_containing_node(&self) -> &Node {
        match self {
            DomContext::Container { container, .. } => container,
            DomContext::Child { parent, .. } | DomContext::Sibling { parent, .. } => {
                parent.get_containing_node()
            }
            #[cfg(test)]
            DomContext::Null => unreachable!(),
        }
    }
    /// Add a new node `new_child` ordered relative to existing siblings according to the given [ChildPosition].
    pub fn add_child(&self, mut position: ChildPosition, new_child: Node) {
        match self {
            DomContext::Container { group, container } => {
                let mut group = group.borrow_mut();
                let reference_sibling = group.range((&position)..).next().map(|(_k, v)| v);
                container
                    .insert_before(&new_child, reference_sibling)
                    .unwrap_throw();
                panic_if_duplicate_node(group.insert(position, new_child));
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
                panic_if_duplicate_node(group.insert(position, new_child));
            }
            DomContext::Child { parent, index } => {
                position.wrap(*index);
                parent.add_child(position, new_child);
            }
            #[cfg(test)]
            DomContext::Null => {}
        }
    }
    /// Remove the child at the given [ChildPosition] and all its descendants.
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
    tree: &mut BTreeMap<ChildPosition, Node>,
    position: ChildPosition,
    container: &Node,
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

#[cfg(debug_assertions)]
fn panic_if_duplicate_node(node: Option<Node>) {
    if let Some(node) = node {
        #[cfg(feature = "csr")]
        {
            web_sys::console::error_2(
                &"Attempted to insert two nodes at the same position.\n\
                You probably either used a `join` implementation from outside Async UI,\
                or tried to render something in a spawned Future.\n\
                This message is only shown in debug builds.\n\
                Check the code where you render this node:\
                "
                .into(),
                node.as_ref(),
            );
        }
        drop(node);
        // TODO: Message in SSR
        panic!()
    }
}
#[cfg(not(debug_assertions))]
fn panic_if_duplicate_node(_node: Option<web_sys::Node>) {}
