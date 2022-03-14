mod position;
use std::{
    future::{pending, Future},
    pin::Pin,
};

use async_ui_spawn::{singlethread::SpawnedFuture, RootSpawnWrapperFuture, SpawnContext};
use position::Position;
use web_sys::Node;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub struct Element<'e>(BoxFuture<'e, ()>);

scoped_tls::scoped_thread_local!(
    pub(crate) static CONTEXT: AUIWContext
);

struct AUIWContext {
    position: Position,
}

impl SpawnContext for AUIWContext {
    fn get_tls() -> &'static scoped_tls::ScopedKey<AUIWContext> {
        &CONTEXT
    }
}

impl<'e, F: Future<Output = ()> + 'e> From<F> for Element<'e> {
    fn from(future: F) -> Self {
        use smol::future::FutureExt;
        Self(future.boxed_local())
    }
}

pub async fn render(children: Vec<Element<'_>>) {
    let mut tasks = Vec::with_capacity(children.len());
    for (index, child) in children.into_iter().enumerate() {
        let context = CONTEXT.with(|ctx| AUIWContext {
            position: ctx.position.nest_fragment(index),
        });

        // SpawnedFuture completes immediately. The await is just to get the Pinning required for safety.
        tasks.push(SpawnedFuture::new(child.0, context).await);
    }

    // await forever so as not to drop tasks
    pending().await
}

pub async fn render_node_and_children(children: Vec<Element<'_>>, node: Node) {
    CONTEXT.with(|ctx| ctx.position.add_node(node.clone()));
    let position = Position::new_in_node(node);
    let mut tasks = Vec::with_capacity(children.len());
    for (index, child) in children.into_iter().enumerate() {
        let context = AUIWContext {
            position: position.nest_fragment(index),
        };
        tasks.push(SpawnedFuture::new(child.0, context).await);
    }
    pending().await
}

pub async fn mount(root: Element<'static>, node: Node) {
    let context = AUIWContext {
        position: Position::new_in_node(node),
    };
    let future = SpawnedFuture::new(root.0, context);
    let _task = RootSpawnWrapperFuture::new(future).await;
    pending().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
