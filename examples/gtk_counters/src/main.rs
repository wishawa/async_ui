use std::time::Duration;

use async_ui_gtk::{mount_and_present, render};
use async_ui_gtk_widgets::Wrappable;
use async_ui_reactive::Rx;
use async_ui_utils::{race, vec_into};
use gtk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    Application, ApplicationWindow,
};

fn main() {
    let app = Application::builder()
        .application_id("org.gtk-rs.example")
        .build();

    let build_ui = |app: &Application| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("My GTK App")
            .build();
        mount_and_present(my_component().into(), window);
    };
    app.connect_activate(build_ui);
    app.run();
}
async fn my_component() {
    race(
        render(vec_into![gtk::Label::new(Some("hello world")).wrap()]),
        async {
            smol::Timer::after(Duration::from_secs(3)).await;
        },
    )
    .await;
    let count = Rx::new(0);
    let label = gtk::Label::new(None);
    race(
        render(vec_into![gtk::Box::new(gtk::Orientation::Vertical, 4)
            .wrap()
            .children(vec_into![
                gtk::Button::with_label("-").wrap().on_clicked(|_| {
                    count.visit_mut(|v| *v -= 1);
                }),
                label.clone().wrap(),
                gtk::Button::with_label("+").wrap().on_clicked(|_| {
                    count.visit_mut(|v| *v += 1);
                }),
            ])]),
        async {
            count
                .for_each(|v| {
                    label.set_text(&v.to_string());
                })
                .await
        },
    )
    .await;
}
