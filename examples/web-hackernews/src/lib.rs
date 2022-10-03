use std::{collections::VecDeque, error::Error, future::pending};

use async_ui_web::{
    components::{
        button, link, list, text, view, ButtonProps, LinkProps, ListModel, ListProps, ViewProps,
    },
    fragment, mount,
};
use observables::cell::ReactiveCell;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    mount(root());
    Ok(())
}

async fn root() {
    async fn root_fallible() -> Result<(), Box<dyn Error>> {
        let client = surf::client();
        let mut ids: VecDeque<u64> = client
            .get("https://hacker-news.firebaseio.com/v0/topstories.json")
            .await?
            .body_json()
            .await?;
        let list_model = ReactiveCell::new(ListModel::from_iter(ids.drain(..40)));
        async fn item(client: &surf::Client, story_id: u64) {
            async fn item_fallible(
                client: &surf::Client,
                story_id: u64,
            ) -> Result<(), Box<dyn Error>> {
                let story: Story = client
                    .get(format!(
                        "https://hacker-news.firebaseio.com/v0/item/{story_id}.json"
                    ))
                    .await?
                    .body_json()
                    .await?;
                link(LinkProps {
                    children: fragment((
                        view(ViewProps {
                            children: fragment((text(&story.title),)),
                            class: Some(&"story-title".into()),
                            ..Default::default()
                        }),
                        view(ViewProps {
                            children: fragment((
                                view(ViewProps {
                                    children: fragment((text(&format!("by: {}", story.by)),)),
                                    class: Some(&"story-author".into()),
                                    ..Default::default()
                                }),
                                view(ViewProps {
                                    children: fragment((text(&format!("{} points", story.score)),)),
                                    ..Default::default()
                                }),
                            )),
                            class: Some(&"story-info-bar".into()),
                            ..Default::default()
                        }),
                    )),
                    class: Some(&"story-item".into()),
                    href: Some(&story.url),
                    ..Default::default()
                })
                .await;
                Ok(())
            }
            item_fallible(client, story_id).await;
            pending::<()>().await;
        }
        fragment((
            list(ListProps {
                data: Some(&list_model.as_observable()),
                render: Some(&|id| item(&client, id)),
                ..Default::default()
            }),
            button(ButtonProps {
                children: fragment((text(&"load more"),)),
                on_press: Some(&mut |_ev| {
                    let mut bm = list_model.borrow_mut();
                    for item in ids.drain(..40) {
                        bm.push(item);
                    }
                }),
                class: Some(&"load-more-button".into()),
                ..Default::default()
            }),
        ))
        .await;
        Ok(())
    }
    root_fallible().await;
    text(&"error").await;
}

#[derive(serde::Deserialize)]
struct Story {
    by: String,
    descendants: usize,
    id: u64,
    title: String,
    time: u64,
    url: String,
    score: i32,
}
