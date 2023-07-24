// ANCHOR: fibo-imports
use async_ui_web::{
    html::Div,
    join,
    lists::{ListModel, ModeledList}, // 👈 new!
    shortcut_traits::ShortcutRenderStr,
};
use gloo_timers::future::TimeoutFuture;
// ANCHOR_END: fibo-imports
// ANCHOR: fibo-helper
async fn render_one_item(n: usize, fib_n: u64) {
    Div::new()
        .render(format!("The {n}th Fibonacci number is {fib_n}.").render())
        .await;
}
// ANCHOR_END: fibo-helper
// ANCHOR: fibo
async fn fibonacci() {
    // 👇 create a list that can render numbers
    let list = ModeledList::new(|(n, fib_n)| render_one_item(*n, *fib_n));
    // 👇 create a model that contains the numbers we'll render
    let mut fibo = ListModel::from(vec![
        (1, 1), // fib_1 is 1
        (2, 1), // fib_2 is also 1
    ]);
    // 👇 tell the list to render the numbers in the `fibo` model
    list.update(&fibo);

    // join 2 Futures:
    // * the list
    // * a Future to manipulate the items
    join((
        list.render(), // 👈 render the list
        async {
            loop {
                // wait 1 second
                TimeoutFuture::new(1000).await;

                // 👇 change `fibo`, adding the next fibonacci number
                fibo.push((
                    fibo.len() + 1, // `n` - the index of the next fibo number
                    // compute `fib_n`
                    fibo.iter()
                        .rev()
                        .map(|(_n, fib_n)| fib_n)
                        .take(2)
                        .cloned()
                        .sum(),
                ));
                // 👇 tell the list that the numbers in the model have changed
                list.update(&fibo);
            }
        },
    ))
    .await;
}
// ANCHOR_END: fibo
