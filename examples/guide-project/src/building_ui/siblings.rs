#[rustfmt::skip]
async fn example() {
use async_ui_web::join;
// ANCHOR: join-example
async fn do_something(input: i32) -> i32 {
	// ...make a network request of something...
	input * 2
}
// Join 2-tuple of Futures
let (res_1, res_2) = join((do_something(21), do_something(100))).await;
assert_eq!(res_1, 42);
assert_eq!(res_2, 200);
// ANCHOR_END: join-example
}

// ANCHOR: two-inputs
use async_ui_web::{html::Input, join}; // 👈 get the `join` function
async fn two_inputs() {
    let input_1 = Input::new();
    let input_2 = Input::new();
    // 👇 join takes a tuple of Futures
    join((input_1.render(), input_2.render())).await;
}
// ANCHOR_END: two-inputs
