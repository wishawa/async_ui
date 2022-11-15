use std::{collections::VecDeque, error::Error, future::pending};

#[cfg(feature = "gtk")]
use async_ui_gtk as async_ui;
#[cfg(feature = "web")]
use async_ui_web as async_ui;

use async_ui::{
    components::{button, list, text, view, ButtonProps, ListModel, ListProps, ViewProps},
    fragment, mount,
};
#[cfg(feature = "web")]
use async_ui_web::components::{link, LinkProps};
use observables::cell::ReactiveCell;

pub async fn root() -> Result<(), Box<dyn Error>> {
    let client = surf::client();
    let mut ids: VecDeque<u64> = client
        .get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .await?
        .body_json()
        .await?;
    let list_model = ReactiveCell::new(ListModel::from_iter(
        ids.drain(..std::cmp::min(40, ids.len())),
    ));
    async fn item(client: &surf::Client, story_id: u64) -> Result<(), Box<dyn Error>> {
        let story: Story = client
            .get(format!(
                "https://hacker-news.firebaseio.com/v0/item/{story_id}.json"
            ))
            .await?
            .body_json()
            .await?;

        #[cfg(feature = "web")]
        use {link as item_wrap, LinkProps as ItemWrapProps};
        #[cfg(feature = "gtk")]
        use {view as item_wrap, ViewProps as ItemWrapProps};

        item_wrap(ItemWrapProps {
            children: fragment((
                view(ViewProps {
                    children: fragment((text(&story.title),)),
                    #[cfg(feature = "web")]
                    class: Some(&"story-title".into()),
                    ..Default::default()
                }),
                view(ViewProps {
                    children: fragment((
                        view(ViewProps {
                            children: fragment((text(&format!("by: {}", story.by)),)),
                            #[cfg(feature = "web")]
                            class: Some(&"story-author".into()),
                            ..Default::default()
                        }),
                        view(ViewProps {
                            children: fragment((text(&format!("{} points", story.score)),)),
                            ..Default::default()
                        }),
                    )),
                    #[cfg(feature = "web")]
                    class: Some(&"story-info-bar".into()),
                    ..Default::default()
                }),
            )),
            #[cfg(feature = "web")]
            class: Some(&"story-item".into()),
            #[cfg(feature = "web")]
            href: Some(&story.url),
            ..Default::default()
        })
        .await;
        Ok(())
    }
    fragment((
        list(ListProps {
            data: Some(&list_model.as_observable()),
            render: Some(&|id| item(&client, id)),
            ..Default::default()
        }),
        button(ButtonProps {
            children: fragment((text(&"Load More Stories"),)),
            on_press: Some(&mut |_ev| {
                let mut bm = list_model.borrow_mut();
                for item in ids.drain(..std::cmp::min(40, ids.len())) {
                    bm.push(item);
                }
            }),
            #[cfg(feature = "web")]
            class: Some(&"load-more-button".into()),
            ..Default::default()
        }),
    ))
    .await;
    Ok(())
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
