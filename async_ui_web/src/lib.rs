mod unmounting;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    future::{pending, Future},
    pin::Pin,
    rc::Rc,
};

use async_ui_spawn::{singlethread::SpawnedFuture, RootSpawnWrappedFuture};
use smallvec::SmallVec;
use web_sys::Node;

pub struct Element<'e>(Box<dyn ElementTrait<'e>>);
pin_project_lite::pin_project! {
struct ElementInner<F>
where F: Future<Output = ()>
{
    parent: ElementParent,
    #[pin]
    future: F
}
}

#[derive(Clone, PartialEq, Eq, Default)]
struct PositionIndices(SmallVec<[usize; 6]>);
impl PartialOrd for PositionIndices {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PositionIndices {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}

type ElementParent = Rc<ElementParentEnum>;
enum ElementParentEnum {
    Node {
        parent_node: Node,
        btree: RefCell<BTreeMap<PositionIndices, Node>>,
    },
    Nest {
        parent: ElementParent,
        index: usize,
    },
    Null,
}
impl ElementParentEnum {
    fn put_node(&self, mut position: PositionIndices, node: Node) {
        match self {
            ElementParentEnum::Node { parent_node, btree } => {
                let mut bm = btree.borrow_mut();
                let next_node = bm.range(position.clone()..).next().map(|(_, v)| v);
                parent_node
                    .insert_before(&node, next_node)
                    .expect("node insertion failed");
                if bm.insert(position, node).is_some() {
                    panic!("put_node more than once");
                }
            }
            ElementParentEnum::Nest { parent, index } => {
                if *index != 0 {
                    position.0.push(*index);
                }
                parent.put_node(position, node)
            }
            ElementParentEnum::Null => panic!("null ElementParent in mounted component"),
        }
    }
    fn remove_node(&self, mut position: PositionIndices) {
        match self {
            ElementParentEnum::Node { parent_node, btree } => {
                let mut bm = btree.borrow_mut();
                let node = bm.remove(&position).expect("to-remove node not found");
                parent_node
                    .remove_child(&node)
                    .expect("node removal failed");
            }
            ElementParentEnum::Nest { parent, index } => {
                if *index != 0 {
                    position.0.push(*index);
                }
                parent.remove_node(position)
            }
            ElementParentEnum::Null => todo!(),
        }
    }
}

scoped_tls::scoped_thread_local! {
    pub(crate) static PARENT: ElementParent
}
trait ElementTrait<'e>: 'e {
    fn to_boxed_future(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + 'e>>;
    fn set_parent(&mut self, parent: ElementParent);
}
impl<'e, F: Future<Output = ()> + 'e> ElementTrait<'e> for ElementInner<F> {
    fn to_boxed_future(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + 'e>> {
        let boxed = self as Box<dyn Future<Output = ()> + 'e>;
        boxed.into()
    }
    fn set_parent(&mut self, parent: ElementParent) {
        self.parent = parent;
    }
}
impl<'e, F: Future<Output = ()> + 'e> Future for ElementInner<F> {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        PARENT.set(this.parent, || this.future.poll(cx))
    }
}

thread_local! {
    static DUMMY_PARENT: ElementParent = Rc::new(ElementParentEnum::Null);
}
impl<'e, F: Future<Output = ()> + 'e> From<F> for Element<'e> {
    fn from(future: F) -> Self {
        let parent = DUMMY_PARENT.with(Clone::clone);
        let inner = ElementInner { parent, future };
        Self(Box::new(inner) as Box<_>)
    }
}

async fn render_inner<'e>(mut children: Vec<Element<'e>>, parent: ElementParent) {
    if children.len() == 1 {
        let mut child = children.pop().unwrap();
        child
            .0
            .set_parent(Rc::new(ElementParentEnum::Nest { index: 0, parent }));
        let _task = SpawnedFuture::new(child.0.to_boxed_future()).await;
        pending().await
    } else {
        let mut tasks = Vec::with_capacity(children.len());
        for (index, mut child) in children.into_iter().enumerate() {
            child.0.set_parent(Rc::new(ElementParentEnum::Nest {
                index: index + 1,
                parent: parent.clone(),
            }));
            tasks.push(SpawnedFuture::new(child.0.to_boxed_future()).await);
        }
        pending().await
    }
}
pub async fn render(children: Vec<Element<'_>>) {
    let parent = PARENT.with(Clone::clone);
    render_inner(children, parent).await
}
pub struct PutNodeGuard {
    parent: ElementParent,
}
impl Drop for PutNodeGuard {
    fn drop(&mut self) {
        self.parent.remove_node(PositionIndices::default());
    }
}

pub fn put_node(node: Node) -> PutNodeGuard {
    let parent = PARENT.with(|parent: &ElementParent| {
        parent.put_node(PositionIndices::default(), node);
        parent.clone()
    });
    PutNodeGuard { parent }
}

pub async fn render_in_node(children: Vec<Element<'_>>, node: Node) {
    let parent = ElementParentEnum::Node {
        btree: RefCell::new(BTreeMap::new()),
        parent_node: node,
    };
    let parent = Rc::new(parent);
    render_inner(children, parent).await
}

pub async fn mount(root: Element<'static>, node: Node) {
    // let context = AUIWContext {
    //     position: Position::new_in_node(node),
    // };
    // let future = SpawnedFuture::new(root.0, context);
    // let _task = RootSpawnWrappedFuture::new(future).await;
    // pending().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
