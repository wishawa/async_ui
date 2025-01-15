
#[cfg(feature = "ssr")]
fn main() {
    use async_ui_web::render_to_string;
    use web_todomvc::app::app;
    let v = render_to_string(app());

    let v = futures_lite::future::block_on(v);
    println!("{v}");
}

#[cfg(not(feature = "ssr"))]
fn main() {
    panic!("ssr requires ssr feature")
}
