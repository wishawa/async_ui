use std::time::Duration;

use async_ui_gtk::{list, mount_and_present, render};
use async_ui_gtk_widgets::Wrappable;
use async_ui_reactive::Rx;
use async_ui_utils::{join2, race2, vec_into};
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
    let label = gtk::Label::new(Some("hello world. please wait 3 secs."));
    let mut remaining_time = 3f32;
    race2(render(vec_into![label.clone().wrap()]), async {
        while remaining_time > 0.0 {
            smol::Timer::after(Duration::from_secs_f32(0.1)).await;
            remaining_time -= 0.1;
            label.set_text(&format!(
                "hello world. please wait {remaining_time:.1} secs."
            ));
        }
    })
    .await;
    let count = Rx::new(0);
    let label = gtk::Label::new(None);
    join2(
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
                list_test()
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
async fn list_test() {
    let children = Rx::new(vec![
        (1, Some(gtk::Label::new(Some("1")).wrap().into())),
        (2, Some(gtk::Label::new(Some("2")).wrap().into())),
        (4, Some(gtk::Label::new(Some("4")).wrap().into())),
    ]);
    join2(
        gtk::Box::new(gtk::Orientation::Horizontal, 4)
            .wrap()
            .children(vec_into![list(&children)]),
        async {
            for top in 10..20 {
                smol::Timer::after(Duration::from_secs(1)).await;
                children.borrow_mut().push((
                    top,
                    Some(gtk::Label::new(Some(&top.to_string())).wrap().into()),
                ));
            }
        },
    )
    .await;
}
