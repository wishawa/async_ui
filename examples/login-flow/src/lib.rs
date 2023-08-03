use async_ui_web::{
    event_traits::EmitElementEvent,
    html::{Button, Input},
    join, mount, race,
    shortcut_traits::{ShortcutRenderStr, UiFutureExt},
};

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    mount(app());
}

async fn app() {
    match connector().await {
        Ok(data) => format!("the connector returned: {data}").render().await,
        Err(_) => "the connector errored :(".render().await,
    }
}

async fn connector() -> Result<String, ()> {
    // loop until successful login
    let token = loop {
        let (username, password) = show_login_form().await;
        if let Ok(token) = login(&username, &password).await {
            break token;
        }
    };
    let list = fetch_list(&token).await?; // 👈 try operator
    let chosen_data_id = show_list(&list).await;
    Ok(get_data(&token, chosen_data_id).await?) // 👈 try operator
}

async fn show_login_form() -> (String, String) {
    let (username, password) = (Input::new(), Input::new_password());
    username.set_placeholder("Username");
    password.set_placeholder("Password");
    let button = Button::new();

    button
        .until_click()
        .meanwhile(join((
            "username is 'user' and password is 'asdf'".render(),
            username.render(),
            password.render(),
            button.render("Login".render()),
        )))
        .await;

    (username.value(), password.value())
}

async fn login(username: &str, password: &str) -> Result<String, ()> {
    // take some time
    gloo_timers::future::TimeoutFuture::new(1500)
        .meanwhile("logging in".render())
        .await;

    match (username, password) {
        ("user", "asdf") => Ok("secrettoken".to_string()),
        _ => {
            let retry_btn = Button::new();
            retry_btn
                .until_click()
                .meanwhile(join((
                    "incorrect username/password".render(),
                    retry_btn.render("Retry".render()),
                )))
                .await;
            Err(())
        }
    }
}

type DataId = i32;
async fn fetch_list(_token: &str) -> Result<Vec<DataId>, ()> {
    // take some time
    gloo_timers::future::TimeoutFuture::new(1500)
        .meanwhile("fetching list of available data".render())
        .await;

    Ok(vec![1, 2, 3, 4])
}

async fn show_list(list: &[DataId]) -> &DataId {
    race((
        "which one do you want?".render().pend_after(),
        race(
            list.iter()
                .map(|id| async move {
                    let btn = Button::new();
                    btn.until_click()
                        .meanwhile(btn.render(format!("data with id {id}").render()))
                        .await;
                    id
                })
                .collect::<Vec<_>>(),
        ),
    ))
    .await
}

async fn get_data(_token: &str, id: &DataId) -> Result<String, ()> {
    // take some time
    gloo_timers::future::TimeoutFuture::new(1500)
        .meanwhile("fetching data".render())
        .await;

    Ok(format!("data id={id}: blah blah blah"))
}
