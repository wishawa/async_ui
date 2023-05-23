use std::{f32::consts::FRAC_PI_6, time::Duration};
use time::Time;

use async_ui_web::{
    components::{Div, Text},
    join,
    prelude_traits::*,
    NoChild,
};

fn now() -> Time {
    time::OffsetDateTime::now_local().unwrap().time()
}
async fn next_second() {
    gloo_timers::future::sleep(Duration::from_millis(1000 - now().millisecond() as u64)).await;
}

pub async fn digital() {
    let time = Text::new();
    let format = time::format_description::parse("[hour]:[minute]:[second]").unwrap();
    join(("Time: ".render(), time.render(), async {
        loop {
            time.set_data(&now().format(&format).unwrap());
            next_second().await;
        }
    }))
    .await;
}

struct AnalogArm {
    div: Div,
}
impl AnalogArm {
    fn new(height_px: u32, width_px: u32) -> Self {
        let div = Div::new();
        div.add_class(style::clock_arm);
        div.style()
            .set_property("height", &format!("{height_px}px"))
            .unwrap();
        div.style()
            .set_property("width", &format!("{width_px}px"))
            .unwrap();
        Self { div }
    }
    async fn render(&self) {
        self.div
            .render(
                {
                    let x = Div::new();
                    x.add_class(style::clock_arm_half);
                    x
                }
                .render(NoChild),
            )
            .await
    }
    fn set_angle(&self, deg: f32) {
        self.div
            .style()
            .set_property("transform", &format!("rotate({}deg)", deg))
            .unwrap();
    }
}

pub async fn analog() {
    async fn numbers() {
        let fut: [_; 12] = core::array::from_fn(|i| {
            let number = Div::new();
            let angle = -((i + 10) as f32 * FRAC_PI_6);
            let top = -angle.sin() * 120.0 + 120.0 - 8.0;
            let left = angle.cos() * 120.0 + 120.0 - 8.0;
            number.add_class(style::clock_number);
            number
                .style()
                .set_property("top", &format!("{top}px"))
                .unwrap();
            number
                .style()
                .set_property("left", &format!("{left}px"))
                .unwrap();
            number.set_inner_text(&format!("{}", i + 1));
            number.render(NoChild)
        });
        join(fut).await;
    }

    let arm_h = AnalogArm::new(100, 8);
    let arm_m = AnalogArm::new(200, 4);
    let arm_s = AnalogArm::new(160, 2);
    join((
        "Clock".render(),
        {
            let x = Div::new();
            x.add_class(style::clock_wrapper);
            x
        }
        .render(join((
            numbers(),
            arm_h.render(),
            arm_m.render(),
            arm_s.render(),
            async {
                loop {
                    let s = (now() - time::Time::MIDNIGHT).as_seconds_f32();
                    arm_h.set_angle(((s / 3600.0) % 12.0) * 30.0);
                    arm_m.set_angle(((s / 60.0) % 60.0) * 6.0);
                    arm_s.set_angle((s % 60.0) * 6.0);
                    next_second().await;
                }
            },
        ))),
    ))
    .await;
}

mod style {
    async_ui_web::css!(
        r#"
.clock-wrapper {
    width: 240px;
    height: 240px;
    position: relative;
}
.clock-number {
    position: absolute;
    width: 16px;
    height: 16px;
}
.clock-arm-half {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    bottom: 50%;
    background-color: red;
    border-radius: 100px;
    overflow: hidden;
}
.clock-arm {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;
    width: {width_px}px;
    height: {height_px}px;
    margin: auto;
}
        "#
    );
}
